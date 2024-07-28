use crate::error::Error;

pub async fn download_file(url: &str) -> Result<String, Error> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(Error::StringError(format!(
            "Failed to fetch file over network: {}",
            response.status()
        )));
    }

    let body = response.text().await?;
    Ok(body)
}
