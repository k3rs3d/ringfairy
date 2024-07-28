use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use std::result::Result;

use crate::cli::AppSettings;
use crate::error::Error;
use crate::file::parse_website_list;
use crate::gen::{html::HtmlGenerator, webring, Generator};
use crate::http::setup_client;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Website {
    pub slug: String,
    pub name: Option<String>,
    pub about: Option<String>,
    pub url: String,
    pub rss: Option<String>,
    pub owner: Option<String>,
}

async fn fetch_website_content(
    client: &reqwest::Client,
    url: &str,
    settings: &AppSettings,
) -> Result<String, Error> {
    for attempt in 1..=settings.audit_retries_max {
        match client.get(url).send().await {
            Ok(response) => match response.text().await {
                Ok(text) => return Ok(text),
                Err(e) => log::debug!("Attempt {}: Failed to read response text: {}", attempt, e),
            },
            Err(e) => log::debug!("Attempt {}: Failed to fetch URL: {}", attempt, e),
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
    let anchor_selector = scraper::Selector::parse("a").map_err(|e| (website.clone(), e.into()))?;
    let button_selector = scraper::Selector::parse("button").map_err(|e| (website.clone(), e.into()))?;
    let img_selector = scraper::Selector::parse("img").map_err(|e| (website.clone(), e.into()))?;

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
