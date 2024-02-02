use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct Website {
    pub name: String,
    pub about: String,
    pub url: String,
    pub owner: String,
}

pub fn verify_websites(websites: &[Website]) -> Result<(), Box<dyn std::error::Error>> {
    let mut names = HashSet::new();
    let mut urls = HashSet::new();
    //let url_pattern = Regex::new(r"^https://.+\..+$")?;

    for website in websites {
        // Check for duplicate names
        if !names.insert(&website.name) {
            return Err(format!("Duplicate website name found: {}", website.name).into());
        }

        // Check for duplicate URLs
        if !urls.insert(&website.url) {
            return Err(format!("Duplicate website URL found: {}", website.url).into());
        }

        // Validate URLs with regex
        /*
        if !url_pattern.is_match(&website.url) {
            return Err(format!("Invalid URL format detected: {}", website.url).into());
        }
        */
    }

    println!("All websites verified successfully.");
    Ok(())
}