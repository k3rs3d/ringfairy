use futures::stream::{FuturesUnordered, StreamExt};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::result::Result;

use crate::cli::AppSettings;
use crate::file;
use crate::html::HtmlGenerator;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Website {
    pub slug: String,
    pub name: Option<String>,
    pub about: Option<String>,
    pub url: String,
    pub rss: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WebringSite {
    pub website: Website,
    pub next: usize,
    pub previous: usize,
}

pub fn setup_client() -> reqwest::Client {
    // TODO: Create a setting/param for timeout duration, and one for user agent string, and one for header string
    // Also, create {{ tags }} for those, so webring admins can show their users those settings
    reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .pool_max_idle_per_host(10)
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.212 Safari/537.36")
    .default_headers({
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".parse().unwrap());
        headers
    })
    .redirect(reqwest::redirect::Policy::limited(5))
    .build()
    .unwrap()
}

pub async fn process_websites(settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    let websites = parse_website_list(&settings.filepath_list).await?;

    // Verify websites entries if required (offline)
    if !settings.skip_verify {
        log::debug!("Verifying websites...");
        verify_websites(&websites)?;
        log::info!("All website entries verified.");
    }

    // Audit websites to ensure they contain webring links (online)
    let audited_websites = if settings.audit {
        log::debug!("Auditing websites for webring links...");
        let audited_websites = audit_links(&setup_client(), websites, &settings.base_url).await?;
        log::info!(
            "Audit complete. Found links on {} websites.",
            audited_websites.len()
        );
        audited_websites
    } else {
        websites
    };

    // Organize site into the webring sequence
    let webring = build_webring_sites(audited_websites, settings.shuffle).await;

    // Proceed with HTML generation (if not a dry run)
    if !settings.dry_run {
        let html_generator = HtmlGenerator::new(&settings.path_templates, settings.skip_minify)?;

        log::info!("Generating websites HTML...");

        html_generator.generate_html(&webring, &settings).await?;

        log::info!("Finished generating webring HTML.");

        log::info!("Generating OPML file...");
        html_generator.generate_opml(&webring, &settings).await?;
    }

    Ok(())
}

// Load the websites from JSON
pub async fn parse_website_list(
    file_path_or_url: &str,
) -> Result<Vec<Website>, Box<dyn std::error::Error>> {
    // Able to get data from local or from remote
    let file_data = file::acquire_file_data(file_path_or_url).await?;

    // Extract file extension to determine the deserialization format
    match file::get_extension_from_path(file_path_or_url).as_deref() {
        Some("json") => {
            // Deserialize JSON
            let websites: Vec<Website> = serde_json::from_str(&file_data)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            Ok(websites)
        }
        // TODO: Support TOML for website lists
        /*
        Some("toml") => {
            // Deserialize TOML
            let websites: Vec<Website> = toml::from_str(&file_data)
                .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
            Ok(websites)
        }
        */
        _ => Err(Box::<dyn std::error::Error>::from(
            "Unsupported file format",
        )),
    }
}

pub fn verify_websites(websites: &[Website]) -> Result<(), Box<dyn std::error::Error>> {
    let mut slugs = HashSet::new();
    let mut urls = HashSet::new();
    // TODO: verify URL pattern
    // let url_pattern = Regex::new(r"^https://.+\..+$")?;

    for website in websites {
        // Check for duplicate names and URLs
        if !slugs.insert(&website.slug) {
            return Err(format!(
                "Duplicate website slug found: {} - {}",
                website.slug,
                website.owner.as_deref().unwrap_or("")
            )
            .into());
        }
        if !urls.insert(&website.url) {
            return Err(format!(
                "Duplicate website URL found: {} - {}",
                website.url,
                website.owner.as_deref().unwrap_or("")
            )
            .into());
        }

        // Uncomment to check URL format with regex
        /*
        if !url_pattern.is_match(&website.url) {
            return Err(format!("Invalid URL format detected: {}", website.url).into());
        }
        */
    }
    Ok(())
}

// TODO: Turn these constants into settings/parameters
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100; // delay between requests

async fn fetch_website_content(
    client: &reqwest::Client,
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut attempts = 0;

    loop {
        attempts += 1;
        match client.get(url).send().await {
            Ok(response) => match response.text().await {
                Ok(text) => return Ok(text),
                Err(e) => log::warn!("Failed to read response text on attempt {}: {}", attempts, e),
            },
            Err(e) => log::warn!("Failed to fetch URL on attempt {}: {}", attempts, e),
        }

        if attempts >= MAX_RETRIES {
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
    }

    Err(format!("Failed to fetch URL {} after {} attempts", url, MAX_RETRIES).into())
}

pub async fn audit_links(
    client: &reqwest::Client,
    websites: Vec<Website>,
    base_url: &str,
) -> Result<Vec<Website>, Box<dyn std::error::Error>> {
    let mut tasks = FuturesUnordered::new();

    for website in websites {
        let website_clone = website.clone();
        let client = client.clone();
        tasks.push(async move { does_html_contain_links(&client, &website_clone, &base_url).await });
    }

    let mut compliant_sites = Vec::new();

    // Collect results - unpacking the tuple inside Ok variant
    while let Some(result) = tasks.next().await {
        match result {
            // Here, we directly get the tuple (website, bool) from Ok
            Ok((website, true, _)) => compliant_sites.push(website),
            Ok((website, false, Some(reason))) => {
                log::warn!("Site failed audit: {} | REASON: {}", website.url, reason)
            }
            Ok((website, false, None)) => log::warn!("Site failed audit: {}", website.url),
            Err(e) => log::error!("Error during site audit: {:?}", e),
        }
    }

    Ok(compliant_sites)
}

async fn does_html_contain_links(
    client: &reqwest::Client,
    website: &Website,
    base_url: &str,
) -> Result<(Website, bool, Option<String>), (Website, Box<dyn std::error::Error>)> {
    // Implement retry mechanism with a delay pattern.
    let html = fetch_website_content(client, &website.url)
        .await
        .map_err(|e| (website.clone(), e.into()))?;

    let document = scraper::Html::parse_document(&html);

    // Define selectors for different elements that could be links.
    let anchor_selector = scraper::Selector::parse("a").unwrap();
    let button_selector = scraper::Selector::parse("button").unwrap();
    let img_selector = scraper::Selector::parse("img").unwrap();

    let next_link = format!("{}/{}/next", base_url.trim_end_matches('/'), website.slug);
    let prev_link = format!(
        "{}/{}/previous",
        base_url.trim_end_matches('/'),
        website.slug
    );

    //log::debug!("Expected next/previous URLs: {}, {}", &next_link, &prev_link);

    let mut next_exists = false;
    let mut previous_exists = false;

    // First, check <a> elements for next/prev links:
    for element in document.select(&anchor_selector) {
        if let Some(href) = element.value().attr("href") {
            log::debug!("Comparing link href: {}", href);

            let href_trimmed = href.trim_end_matches('/');
            if href_trimmed == next_link {
                next_exists = true;
            } else if href_trimmed == prev_link {
                previous_exists = true;
            }
        }
    }

    // If one/both <a> links are missing, check for buttons
    if !next_exists || !previous_exists {
        for element in document.select(&button_selector) {
            if let Some(onclick) = element.value().attr("onclick") {
                log::debug!("Checking button onclick: {}", onclick);
                if onclick.contains(&next_link) {
                    next_exists = true;
                } else if onclick.contains(&prev_link) {
                    previous_exists = true;
                }
            }
        }
    }

    // Finally, if needed, also check <img> tags with `onclick` attribute
    if !next_exists || !previous_exists {
        for element in document.select(&img_selector) {
            if let Some(onclick) = element.value().attr("onclick") {
                log::debug!("Checking img onclick: {}", onclick);
                if onclick.contains(&next_link) {
                    next_exists = true;
                } else if onclick.contains(&prev_link) {
                    previous_exists = true;
                }
            }
        }
    }

    let result = next_exists && previous_exists;
    let failure_reason = if !result {
        let mut reason = String::new();
        if !next_exists {
            reason += "Missing next link. ";
        }
        if !previous_exists {
            reason += "Missing previous link. ";
        }
        Some(reason)
    } else {
        None
    };

    Ok((website.clone(), result, failure_reason))
}

async fn build_webring_sites(websites: Vec<Website>, shuffle: bool) -> Vec<WebringSite> {
    // Shuffle first (if set to do so)
    let mut websites = websites;
    if shuffle {
        log::info!("Shuffling website sequence...");
        let mut rng = thread_rng(); // shuffle needs an RNG
        websites.as_mut_slice().shuffle(&mut rng);
    }

    let websites_len = websites.len(); // Capture length before consuming vector
    let mut webring_sites: Vec<WebringSite> = Vec::with_capacity(websites_len);

    for (index, website) in websites.into_iter().enumerate() {
        let next_index = if index + 1 == websites_len {
            0
        } else {
            index + 1
        };
        let prev_index = if index == 0 {
            websites_len - 1
        } else {
            index - 1
        };

        webring_sites.push(WebringSite {
            website,
            next: next_index,
            previous: prev_index,
        });
    }

    webring_sites
}
