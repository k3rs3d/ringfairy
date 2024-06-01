use crate::http::download_file;
use std::error::Error;
use std::fs;

pub async fn acquire_file_data(path_or_url: &str) -> Result<String, Box<dyn Error>> {
    // Check if the path_or_url is likely a URL by looking for a scheme
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        // It's a URL, download the file
        download_file(path_or_url).await
    } else {
        // Assume it's a local file path
        match fs::read_to_string(path_or_url) {
            Ok(contents) => Ok(contents),
            Err(e) => Err(format!("Failed to open file: {}", e).into()),
        }
    }
}

pub fn copy_asset_files(
    source_dir: &str,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the target directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Iterate over each entry in the assets directory
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let target_path = std::path::Path::new(output_dir).join(file_name);

        // If the entry is a file, copy it to the output directory
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), target_path)?;
        }
    }

    Ok(())
}

pub fn get_extension_from_path(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}
