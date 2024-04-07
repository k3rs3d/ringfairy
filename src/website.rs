use futures::stream::{FuturesUnordered, StreamExt};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::result::Result;

use crate::file;
use crate::cli::AppSettings;
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
        let audited_websites = audit_links(websites, &settings.base_url).await?;
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

        html_generator
            .generate_html(&webring, &settings.path_output)
            .await?;

        log::info!("Finished generating webring HTML.");
    }

    Ok(())
}

// Load the websites from JSON
pub async fn parse_website_list(
    file_path_or_url: &str,
) -> Result<Vec<Website>, Box<dyn std::error::Error>> {
    // Able to get data from local or from remote
    let file_data = file::acquire_file_data(file_path_or_url).await?;

    // Parse JSON contents from the string
    let websites: Vec<Website> = serde_json::from_str(&file_data)
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    Ok(websites)
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

async fn fetch_website_content(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.212 Safari/537.36")
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(reqwest::header::ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".parse().unwrap());
            headers
        })
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let response = client.get(url).send().await?.error_for_status()?; // will return Err if status is not in 200-299

    if !response.status().is_success() {
        log::error!(
            "Failed to fetch site content: HTTP status {} for {}",
            response.status(),
            url
        );
    }

    Ok(response.text().await?)
}

pub async fn audit_links(
    websites: Vec<Website>,
    base_url: &str,
) -> Result<Vec<Website>, Box<dyn std::error::Error>> {
    let mut tasks = FuturesUnordered::new();

    for website in websites {
        let website_clone = website.clone();

        tasks.push(async move {
            does_html_contain_links(&website_clone, &base_url)
                .await
        });
    }

    let mut compliant_sites = Vec::new();

    // Collect results - unpacking the tuple inside Ok variant
    while let Some(result) = tasks.next().await {
        match result {
            // Here, we directly get the tuple (website, bool) from Ok
            Ok((website, true, _)) => compliant_sites.push(website),
            Ok((website, false, Some(reason))) => log::warn!("Site failed audit: {} | REASON: {}", website.url, reason),
            Ok((website, false, None)) => log::warn!("Site failed audit: {}", website.url),
            Err(e) => log::error!("Error during site audit: {:?}", e),
        }
    }

    Ok(compliant_sites)
}

async fn does_html_contain_links(
    website: &Website,
    base_url: &str,
) -> Result<(Website, bool, Option<String>), (Website, Box<dyn std::error::Error>)> {
    let html = fetch_website_content(&website.url)
        .await
        .map_err(|e| (website.clone(), e.into()))?;
    let document = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse("a").unwrap();

    let next_link = match normalize_url(base_url, &format!("{}/{}/next", base_url.trim_end_matches('/'), website.slug)) {
        Ok(url) => url,
        Err(e) => return Err((website.clone(), e)),
    };
    let prev_link = match normalize_url(base_url, &format!("{}/{}/previous", base_url.trim_end_matches('/'), website.slug)) {
        Ok(url) => url,
        Err(e) => return Err((website.clone(), e)),
    };

    let mut next_exists = false;
    let mut previous_exists = false;

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            let normalized_href = match normalize_url(base_url, href) {
                Ok(url) => url,
                Err(e) => return Err((website.clone(), e)),
            };

            if normalized_href == next_link || normalized_href == prev_link {
                log::debug!("Found matching link for {}", website.slug); 
                if href.contains("/next") {
                    next_exists = true;
                } else if href.contains("/previous") {
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

fn normalize_url(base_url: &str, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Try to parse the URL (If it's an absolute URL, this will succeed)
    // otherwise this will fail and we'll attempt to resolve against the base URL
    // Double slashes are technically valid, replace with single slash to normalize
    let processed_url = url.replace("//", "/").to_owned();
    let parsed_url_result = reqwest::Url::parse(processed_url.as_str());
    
    let mut url = match parsed_url_result {
        Ok(url) => url, // If parsing succeeded, it's absolute
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            // parsing failed because of a relative URL - resolve it against the base URL
            let base = reqwest::Url::parse(base_url)?;
            base.join(url)?
        },
        Err(e) => return Err(e.into()), // For any other error, return it
    };

    url.set_query(None);
    url.set_fragment(None);
    let normalized = url.as_str().trim_end_matches('/').to_string();

    Ok(normalized)
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
