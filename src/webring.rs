use std::collections::HashSet;
use rand::{seq::SliceRandom, thread_rng};

use crate::website::Website;

#[derive(Debug, serde::Serialize)]
pub struct WebringSite {
    pub website: Website,
    pub next: usize,
    pub previous: usize,
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

pub async fn build_webring_sites(websites: Vec<Website>, shuffle: bool) -> Vec<WebringSite> {
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
