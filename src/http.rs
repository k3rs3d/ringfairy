use std::error::Error;
use reqwest;

pub async fn download_file(url: &str) -> Result<String, Box<dyn Error>> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch file over network: {}", response.status()).into());
    }

    let body = response.text().await?;
    Ok(body)
}