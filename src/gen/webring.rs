use rand::{seq::SliceRandom, thread_rng};
use regex::Regex;
use std::collections::HashSet;

use crate::cli::AppSettings;
use crate::error::Error;
use crate::file::parse_website_list;
use crate::gen::{html::HtmlGenerator, Generator};
use crate::http::setup_client;
use crate::website::{audit_links, Website};

#[derive(Debug, serde::Serialize)]
pub struct WebringSite {
    pub website: Website,
    pub next: usize,
    pub previous: usize,
}

/// Checks each Website to ensure it has a valid URL, and tries to detect duplicate entries.
pub fn verify_websites(websites: &[Website]) -> Result<(), Error> {
    let mut slugs = HashSet::new();
    let mut urls = HashSet::new();

    let url_pattern = Regex::new(r"^(http|https)://[^\s/$.?#].[^\s]*$")
        .map_err(|e| Error::StringError(e.to_string()))?;

    for website in websites {
        // Check for invalid URL format
        if !url_pattern.is_match(&website.url) {
            return Err(Error::StringError(format!(
                "Unrecognized URL format: {} - {}",
                website.url, website.slug
            )));
        }
        // Check for duplicate names and URLs
        if !slugs.insert(&website.slug) {
            return Err(Error::StringError(format!(
                "Duplicate website slug found: {} - {}",
                website.slug,
                website.owner.as_deref().unwrap_or("")
            )));
        }
        if !urls.insert(&website.url) {
            return Err(Error::StringError(format!(
                "Duplicate website URL found: {} - {}",
                website.url,
                website.owner.as_deref().unwrap_or("")
            )));
        }
    }
    Ok(())
}

/// Takes the vec of Websites, and outputs an ordered vec of WebringSites  
pub async fn build_webring_sequence(
    websites: Vec<Website>,
    settings: &AppSettings,
) -> Vec<WebringSite> {
    // Shuffle first (if set to do so)
    let mut websites = websites;
    if settings.shuffle {
        log::info!("Shuffling website sequence...");
        let mut rng = thread_rng(); // shuffle needs an RNG
        websites.as_mut_slice().shuffle(&mut rng);
    }

    for (index, website) in websites.iter_mut().enumerate() {
        if settings.no_slug {
            // determine sequential slugs if no_slug is true
            website.slug = (index + 1).to_string();
        } else if website.slug.is_empty() {
            // if slug is missing, derive it from url (minus punctuation)
            website.slug = website.url.replace(|c: char| !c.is_alphanumeric(), "");
        }
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

/// Based on the provided settings, tries to load a list of websites, then generate & save files to create the webring.
pub async fn generate_webring_files(settings: &AppSettings) -> Result<(), Error> {
    let websites = parse_website_list(&settings.filepath_list).await?;

    // Verify websites entries if required (offline)
    if !settings.skip_verify {
        log::info!("Verifying sites...");
        verify_websites(&websites)?;
        log::info!("All site entries verified.");
    }

    // Audit websites to ensure they contain webring links (online)
    let client = setup_client(settings).await?;
    let audited_websites = if settings.audit {
        let websites_len = websites.len(); // capture length of website list
        log::info!("Auditing sites for webring links...");
        let audited_websites = audit_links(&client, websites.clone(), settings).await?;
        log::info!(
            "Audit complete. Detected links on {} out of {} sites.",
            audited_websites.len(),
            websites_len
        );
        audited_websites
    } else {
        websites
    };

    // Ensure the list isn't empty at this point
    if audited_websites.is_empty() {
        return Err(Error::StringError(
            "No valid sites passed the audit.".to_string(),
        ));
    }

    // Organize sites into the webring sequence
    let webring = build_webring_sequence(audited_websites, settings).await;

    // Proceed with HTML generation (if not a dry run)
    if !settings.dry_run {
        log::info!("Generating webring HTML...");
        let html_generator =
            HtmlGenerator::new(settings.path_templates.clone().into(), settings.skip_minify)
                .await?;
        html_generator.generate_content(&webring, settings).await?;
        log::info!("Finished generating webring HTML.");
        //html_generator.generate_opml(&webring, &settings).await?;
    }

    Ok(())
}
