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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_website(slug: &str) -> Website {
        Website {
            slug: slug.to_string(),
            name: Some(format!("Site {}", slug)),
            about: Some(format!("About {}", slug)),
            url: format!("http://{}.com", slug),
            rss: Some(format!("http://{}.com/rss", slug)),
            owner: Some(format!("Owner {}", slug)),
        }
    }

    #[tokio::test]
    async fn test_build_webring() {
        // sample data
        let websites = vec![
            create_sample_website("site1"),
            create_sample_website("site2"),
            create_sample_website("site3"),
        ];

        let webring_sites = build_webring_sites(websites.clone(), false).await;

        assert_eq!(webring_sites.len(), 3);

        assert_eq!(webring_sites[0].website.slug, "site1");
        assert_eq!(webring_sites[0].next, 1);
        assert_eq!(webring_sites[0].previous, 2);

        assert_eq!(webring_sites[1].website.slug, "site2");
        assert_eq!(webring_sites[1].next, 2);
        assert_eq!(webring_sites[1].previous, 0);

        assert_eq!(webring_sites[2].website.slug, "site3");
        assert_eq!(webring_sites[2].next, 0);
        assert_eq!(webring_sites[2].previous, 1);
    }

    #[tokio::test]
    async fn test_build_webring_shuffle() {
        // sample data
        let websites = vec![
            create_sample_website("site1"),
            create_sample_website("site2"),
            create_sample_website("site3"),
        ];

        // shuffle enabled
        let webring_sites = build_webring_sites(websites.clone(), true).await;

        assert_eq!(webring_sites.len(), 3);

        let mut slugs: Vec<&str> = webring_sites.iter().map(|site| site.website.slug.as_str()).collect();
        slugs.sort();
        let expected_slugs: Vec<&str> = websites.iter().map(|site| site.slug.as_str()).collect();
        assert_eq!(slugs, expected_slugs);

        for i in 0..webring_sites.len() {
            let next_index = (i + 1) % webring_sites.len();
            assert_eq!(webring_sites[i].next, next_index);

            let prev_index = if i == 0 {
                webring_sites.len() - 1
            } else {
                i - 1
            };
            assert_eq!(webring_sites[i].previous, prev_index);
        }
    }
}