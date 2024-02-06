use rand::{seq::SliceRandom, thread_rng};
use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::result::Result;

use crate::html::HtmlGenerator; 
use crate::cli::AppSettings; 
use crate::file;

#[derive(Debug, Deserialize, Serialize)]
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

    // Verify websites if required
    if !settings.skip_verify {
        log::debug!("Verifying websites...");
        verify_websites(&websites)?;
        log::info!("All websites verified.");
    }

    // Organize site into the webring sequence
    let webring = build_webring_sites(websites, settings.shuffle).await;

    // Proceed with HTML generation (if not a dry run)
    if !settings.dry_run {
        let html_generator = HtmlGenerator::new(&settings.path_templates, settings.skip_minify)?;

        log::info!("Generating websites HTML...");

        html_generator.generate_html(
            &webring,
            &settings.path_output,
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
    let mut slugs = HashSet::new();
    let mut urls = HashSet::new();
    // let url_pattern = Regex::new(r"^https://.+\..+$")?;

    for website in websites {
        // Check for duplicate names and URLs
        if !slugs.insert(&website.slug) {
            return Err(format!("Duplicate website slug found: {} - {}", website.slug, website.owner.as_deref().unwrap_or("")).into());
        }
        if !urls.insert(&website.url) {
            return Err(format!("Duplicate website URL found: {} - {}", website.url, website.owner.as_deref().unwrap_or("")).into());
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

async fn build_webring_sites(websites: Vec<Website>, shuffle: bool) -> Vec<WebringSite> {
    // Shuffle first (if set to do so)
    let mut websites = websites; // Make mutable
    if shuffle {
        log::info!("Shuffling website sequence...");
        let mut rng = thread_rng(); // shuffle needs an RNG
        websites.as_mut_slice().shuffle(&mut rng); 
    }
    
    let websites_len = websites.len(); // Capture length before consuming vector
    let mut webring_sites: Vec<WebringSite> = Vec::with_capacity(websites_len);
    
    for (index, website) in websites.into_iter().enumerate() {
        let next_index = if index + 1 == websites_len { 0 } else { index + 1 }; 
        let prev_index = if index == 0 { websites_len - 1 } else { index - 1 }; 

        webring_sites.push(WebringSite {
            website,
            next: next_index,
            previous: prev_index,
        });
    }

    webring_sites
}
