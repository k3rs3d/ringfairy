use lazy_static::lazy_static;
use rand::prelude::SliceRandom;
use regex::Regex;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};

pub mod html;
#[cfg(test)]
mod tests;
pub mod webring;

use crate::cli::AppSettings;
use crate::error::Error;
use crate::file::copy_asset_files;
use crate::gen::webring::WebringSite;

///Entry point (for now)
pub async fn make_ringfairy_go_now(settings: &AppSettings) -> Result<(), Error> {
    // Do webring
    webring::generate_webring_files(settings).await?;

    // Copy static files (from ./assets by default) into output folder
    copy_asset_files(&settings.path_assets, &settings.path_output).await?;

    Ok(())
}

/// Generic page generator
pub trait Generator: Send + Sync {
    async fn new(template_path: PathBuf, skip_minify: bool) -> Result<Self, Error>
    where
        Self: Sized;

    async fn write_content(&self, file_path: &Path, content: &str) -> Result<(), Error>;

    async fn generate_content(
        &self,
        webring: &[WebringSite],
        settings: &AppSettings,
    ) -> Result<(), Error>;

    async fn ensure_output_directory(&self, path_output: &str) -> Result<(), Error> {
        fs::create_dir_all(path_output)?;
        Ok(())
    }

    async fn precompute_tags(webring: &[WebringSite], settings: &AppSettings) -> PrecomputedTags {
        let featured_site = webring.choose(&mut rand::thread_rng()).unwrap();

        PrecomputedTags {
            number_of_sites: webring.len(),
            current_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            featured_site_name: featured_site
                .website
                .name
                .clone()
                .unwrap_or_else(|| featured_site.website.url.clone()),
            featured_site_description: featured_site.website.about.clone().unwrap_or_default(),
            featured_site_url: featured_site.website.url.clone(),
            opml_link: format!("./{}.opml", &settings.ring_name),
        }
    }
}

/// Struct for holding precomputed tag data
pub struct PrecomputedTags {
    pub number_of_sites: usize,
    pub current_time: String,
    pub featured_site_name: String,
    pub featured_site_description: String,
    pub featured_site_url: String,
    pub opml_link: String,
}

lazy_static! {
    static ref HYPERLINK_REGEX: Regex =
        Regex::new(r#"<a\s+[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#).unwrap();
    static ref URL_REGEX: Regex = Regex::new(r"^[a-z]+://").unwrap();
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap();
    static ref FEDIVERSE_REGEX: Regex = Regex::new(r"^@([^\s@]+)@([^\s@]+\.[^\s@]+)$").unwrap();
    static ref PHONE_REGEX: Regex = Regex::new(r"^\+?\d{10,15}$").unwrap();
    static ref SMS_REGEX: Regex = Regex::new(r"^sms:\+?\d{10,15}$").unwrap();
}
