use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use std::result::Result;

use crate::cli::AppSettings;
use crate::error::Error;
use crate::file;
use crate::gen::{html::HtmlGenerator, webring, Generator};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Website {
    pub slug: String,
    pub name: Option<String>,
    pub about: Option<String>,
    pub url: String,
    pub rss: Option<String>,
    pub owner: Option<String>,
}

pub fn setup_client(settings: &AppSettings) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .user_agent(settings.client_user_agent.clone())
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::ACCEPT,
                settings.client_header.clone().parse().unwrap(),
            );
            headers
        })
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .unwrap()
}

pub async fn process_websites(settings: &AppSettings) -> Result<(), Error> {
    let websites = parse_website_list(&settings.filepath_list).await?;

    // Verify websites entries if required (offline)
    if !settings.skip_verify {
        log::debug!("Verifying sites...");
        webring::verify_websites(&websites)?;
        log::info!("All site entries verified.");
    }

    // Audit websites to ensure they contain webring links (online)
    let audited_websites = if settings.audit {
        let websites_len = &websites.len(); // capture length of website list
        log::debug!("Auditing sites for webring links...");
        let audited_websites =
            audit_links(&setup_client(&settings), websites.clone(), &settings).await?;
        log::info!(
            "Audit complete. Found links on {} out of {} sites.",
            audited_websites.len(),
            websites_len
        );
        audited_websites
    } else {
        websites
    };

    // Ensure the list isn't empty at this point
    if audited_websites.is_empty() {
        return Err(Error::StringError("No valid sites passed the audit.".to_string()));
    }

    // Organize sites into the webring sequence
    let webring = webring::build_webring_sites(audited_websites, settings).await;

    // Proceed with HTML generation (if not a dry run)
    if !settings.dry_run {
        log::info!("Generating webring HTML...");
        let html_generator =
            HtmlGenerator::new(settings.path_templates.clone().into(), settings.skip_minify)
                .await?;
        html_generator.generate_content(&webring, &settings).await?;
        log::info!("Finished generating webring HTML.");
        //html_generator.generate_opml(&webring, &settings).await?;
    }

    Ok(())
}

// Load the websites from JSON
pub async fn parse_website_list(file_path_or_url: &str) -> Result<Vec<Website>, Error> {
    // Able to get data from local or from remote
    let file_data = file::acquire_file_data(file_path_or_url).await?;

    // Extract file extension to determine the deserialization format
    match file::get_extension_from_path(file_path_or_url).as_deref() {
        Some("json") => {
            // Deserialize JSON
            let websites: Vec<Website> = serde_json::from_str(&file_data)?;
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
        _ => Err(Error::StringError("Unsupported file format".to_string())),
    }
}

async fn fetch_website_content(
    client: &reqwest::Client,
    url: &str,
    settings: &AppSettings,
) -> Result<String, Error> {
    let mut attempts = 0;

    loop {
        attempts += 1;
        match client.get(url).send().await {
            Ok(response) => match response.text().await {
                Ok(text) => return Ok(text),
                Err(e) => log::warn!(
                    "Failed to read response text on attempt {}: {}",
                    attempts,
                    e
                ),
            },
            Err(e) => log::warn!("Failed to fetch URL on attempt {}: {}", attempts, e),
        }

        if attempts >= settings.audit_retries_max {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(
            settings.audit_retries_delay,
        ))
        .await;
    }

    Err(Error::StringError(format!(
        "Failed to fetch {} after {} attempts",
        url, settings.audit_retries_max
    )))
}

pub async fn audit_links(
    client: &reqwest::Client,
    websites: Vec<Website>,
    settings: &AppSettings,
) -> Result<Vec<Website>, Error> {
    let mut tasks = FuturesUnordered::new();

    for website in websites {
        let website_clone = website.clone();
        let client = client.clone();
        tasks
            .push(async move { does_html_contain_links(&client, &website_clone, &settings).await });
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
    settings: &AppSettings,
) -> Result<(Website, bool, Option<String>), (Website, Box<dyn std::error::Error>)> {
    // Implement retry mechanism with a delay pattern.
    let html = fetch_website_content(client, &website.url, settings)
        .await
        .map_err(|e| (website.clone(), e.into()))?;

    let document = scraper::Html::parse_document(&html);

    // Define selectors for different elements that could be links.
    let anchor_selector = scraper::Selector::parse("a").unwrap();
    let button_selector = scraper::Selector::parse("button").unwrap();
    let img_selector = scraper::Selector::parse("img").unwrap();

    let next_link = format!(
        "{}/{}/{}",
        settings.base_url.trim_end_matches('/'),
        website.slug,
        settings.next_url_text
    );
    let prev_link: String = format!(
        "{}/{}/{}",
        settings.base_url.trim_end_matches('/'),
        website.slug,
        settings.prev_url_text
    );

    //log::debug!("Expected next/previous URLs: {}, {}", &next_link, &prev_link);

    let mut next_exists = false;
    let mut previous_exists = false;

    // First, check <a> elements for next/prev links:
    for element in document.select(&anchor_selector) {
        if let Some(href) = element.value().attr("href") {
            log::trace!("Comparing link href: {}", href);

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
                log::trace!("Checking button onclick: {}", onclick);
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
                log::trace!("Checking img onclick: {}", onclick);
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
