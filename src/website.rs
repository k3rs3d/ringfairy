use serde::Deserialize;
use std::collections::HashSet;

use crate::file;

#[derive(Debug, Deserialize)]
pub struct Website {
    pub name: String,
    pub about: String,
    pub url: String,
    pub owner: String,
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
