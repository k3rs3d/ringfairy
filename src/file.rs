use crate::cli::AppSettings;
use crate::error::Error;
use crate::http::download_file;
use crate::website::Website;
use std::fs;
use std::path::Path;

fn parse_csv_websites(csv_data: &str) -> Result<Vec<Website>, Error> {
    let mut rdr = csv::Reader::from_reader(csv_data.as_bytes());
    let mut websites = Vec::new();
    for result in rdr.deserialize() {
        let website: Website = result
            .map_err(|e| Error::StringError(format!("Failed to parse CSV row: {}", e)))?;
        websites.push(website);
    }
    Ok(websites)
}

/// Loads the given file(s) with acquire_file_data(), returns a vec of Websites for each site in the file
pub async fn parse_website_list(settings: &AppSettings) -> Result<Vec<Website>, Error> {
    let mut all_websites = Vec::new();

    // JSON literals
    for json in &settings.json_lists {
        let mut list: Vec<Website> = serde_json::from_str(json)
            .map_err(|e| Error::StringError(format!("Failed to parse JSON literal: {e}")))?;
        all_websites.append(&mut list);
    }

    // TOML literals
    for toml in &settings.toml_lists {
        let mut list: Vec<Website> = toml::from_str(toml)
            .map_err(|e| Error::StringError(format!("Failed to parse TOML literal: {e}")))?;
        all_websites.append(&mut list);
    }

    // Load file(s)
    for path in &settings.filepath_list {
        let file_data = acquire_file_data(path).await?;
        let ext = get_extension_from_path(path).unwrap_or_else(|| "json".into());
        let mut list = match ext.as_str() {
            "json" => serde_json::from_str::<Vec<Website>>(&file_data)
                .map_err(|e| Error::StringError(format!("Failed to parse JSON file '{}': {}", path, e)))?,
            "toml" => toml::from_str::<Vec<Website>>(&file_data)
                .map_err(|e| Error::StringError(format!("Failed to parse TOML file '{}': {}", path, e)))?,
            "csv" => parse_csv_websites(&file_data)
                .map_err(|e| Error::StringError(format!("Failed to parse CSV file '{}': {}", path, e)))?,
            other => return Err(Error::StringError(format!("Unsupported file format '{}'", other))),
        };
        all_websites.append(&mut list);
    }

    Ok(all_websites)
}

/// This will either read or download the file, depending on whether a URL or local URI is provided.
pub async fn acquire_file_data(path_or_url: &str) -> Result<String, Error> {
    // Check if the path_or_url is likely a URL by looking for a scheme
    // TODO: switch this to regex??
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        // It's a URL, download the file
        Ok(download_file(path_or_url).await?)
    } else {
        // Otherwise assume it's a local file path
        Ok(fs::read_to_string(path_or_url)?)
    }
}

pub async fn copy_asset_files(source_dir: &str, output_dir: &str) -> Result<(), Error> {
    // Create the target directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Iterate over each entry in the assets directory
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let target_path = Path::new(output_dir).join(file_name);

        // If the entry is a file, copy it to the output directory
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), target_path)?;
        }
    }

    Ok(())
}

/// Takes a filepath and returns the extension alone. So 'example.jpg' would return 'jpg'.
pub fn get_extension_from_path(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_acquire_file_from_invalid_url() {
        let result = acquire_file_data("http://").await;
        assert!(result.is_err(), "Expected error (invalid URL)");
    }

    #[tokio::test]
    async fn test_acquire_nonexistent_file() {
        let result = acquire_file_data("/path/to/a/nonexistent/file.txt").await;
        assert!(result.is_err(), "Expected error (nonexistent file)");
    }

    #[tokio::test]
    async fn test_acquire_file_with_empty_string() {
        let result = acquire_file_data("").await;
        assert!(result.is_err(), "Expected error (empty filepath string)");
    }

    // get_extension_from_path()
    #[tokio::test]
    async fn test_get_extension_from_valid_path() {
        let path = "file.txt";
        let result = get_extension_from_path(path);
        assert_eq!(result, Some("txt".to_string()));
    }

    #[tokio::test]
    async fn test_get_extension_from_path_without_extension() {
        let path = "file";
        let result = get_extension_from_path(path);
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_get_extension_from_path_with_multiple_dots() {
        let path = "archive.tar.gz";
        let result = get_extension_from_path(path);
        assert_eq!(result, Some("gz".to_string()));
    }
}
