use serde::Deserialize;
use std::collections::HashSet;
use std::result::Result;

use crate::html::HtmlGenerator; 
use crate::cli::AppSettings; 
use crate::file;

#[derive(Debug, Deserialize)]
pub struct Website {
    pub name: String,
    pub about: String,
    pub url: String,
    pub owner: String,
}

#[derive(Debug)]
pub struct WebringSite {
    pub website: Website,
    pub next: usize, 
    pub previous: usize, 
}

pub async fn process_websites(settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    let websites = parse_website_list(&settings.filepath_list).await?;

    // Verify websites if required
    if !settings.skip_verify {
        log::debug!("Verifying websites...");
        verify_websites(&websites)?;
        log::info!("All websites verified.");
    }

    // Organize site into the webring sequence
    let webring = build_webring_sites(websites).await;

    // Proceed with HTML generation (if not a dry run)
    if !settings.dry_run {
        let html_generator = HtmlGenerator::new(settings.skip_minify);

        log::info!("Generating websites HTML...");

        html_generator.generate_websites_html(
            &webring,
            &settings.path_output,
            &settings.filepath_template_redirect,
            &settings.filepath_template_index,
        )
        .await?;
    
        log::info!("Finished generating webring HTML.");
    }

    Ok(())
}

// Load the websites from JSON
pub async fn parse_website_list(file_path_or_url: &str) -> Result<Vec<Website>, Box<dyn std::error::Error>> {
    // Able to get data from local or from remote 
    let file_data = file::acquire_file_data(file_path_or_url).await?;

    // Parse JSON contents from the string
    let websites: Vec<Website> = serde_json::from_str(&file_data)
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    Ok(websites)
}

pub fn verify_websites(
    websites: &[Website],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut names = HashSet::new();
    let mut urls = HashSet::new();
    // let url_pattern = Regex::new(r"^https://.+\..+$")?;

    for website in websites {
        // Check for duplicate names and URLs
        if !names.insert(&website.name) {
            return Err(format!("Duplicate website name found: {} - {}", website.name, website.owner).into());
        }
        if !urls.insert(&website.url) {
            return Err(format!("Duplicate website URL found: {} - {}", website.url, website.owner).into());
        }

        // Uncomment to check URL format (needs regex set up)
        /*
        if !url_pattern.is_match(&website.url) {
            return Err(format!("Invalid URL format detected: {}", website.url).into());
        }
        */
    }
    Ok(())
}

async fn build_webring_sites(websites: Vec<Website>) -> Vec<WebringSite> {
    let websites_len = websites.len(); // Capture length before consuming vector
    let mut webring_sites: Vec<WebringSite> = Vec::with_capacity(websites_len);
    
    for (index, website) in websites.into_iter().enumerate() {
        let next_index = if index + 1 == websites_len { 0 } else { index + 1 }; // Use captured length
        let prev_index = if index == 0 { websites_len - 1 } else { index - 1 }; // Use captured length

        webring_sites.push(WebringSite {
            website,
            next: next_index,
            previous: prev_index,
        });
    }

    webring_sites
}
