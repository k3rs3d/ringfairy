use std::fs;
use std::error::Error;
use crate::http;

pub async fn acquire_file_data(path_or_url: &str) -> Result<String, Box<dyn Error>> {
    // Check if the path_or_url is likely a URL by looking for a scheme
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        // It's a URL, download the file
        http::download_file(path_or_url).await
    } else {
        // Assume it's a local file path
        match fs::read_to_string(path_or_url) {
            Ok(contents) => Ok(contents),
            Err(e) => Err(format!("Failed to open file: {}", e).into()),
        }
    }
}

pub fn copy_asset_files() -> Result<(), Box<dyn std::error::Error>> {
    let source_dir = "assets";
    let target_dir = "webring";

    // Create the target directory if it doesn't exist
    fs::create_dir_all(target_dir)?;

    // Iterate over each entry in the source directory
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let target_path = std::path::Path::new(target_dir).join(file_name);

        // If the entry is a file, copy it to the target directory
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), target_path)?;
        }
    }

    Ok(())
}