use crate::cli::AppSettings;
use crate::error::Error;

/// Returns a reqwest client, configured according to the user's configuration settings
pub async fn setup_client(settings: &AppSettings) -> Result<reqwest::Client, Error> {
    log::trace!("Building reqwest client...");
    Ok(reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .pool_max_idle_per_host(10)
        .user_agent(settings.client_user_agent.clone())
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::ACCEPT,
                settings.client_header.clone().parse().unwrap(),
            );
            headers
        })
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?)
}

/// Returns a string from the contents of the given URL (using a temporary/internal reqwest client)
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
