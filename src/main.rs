mod cli;
mod file;
mod html;
mod http;
mod website;

use crate::website::Website;

// Load the websites from JSON
async fn parse_website_list(file_path_or_url: &str) -> Result<Vec<Website>, Box<dyn std::error::Error>> {
    // Use the abstract function to acquire data
    let file_data = file::acquire_file_data(file_path_or_url).await?;

    // Parse JSON contents from the string
    let websites: Vec<Website> = serde_json::from_str(&file_data)
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    Ok(websites)
}

#[tokio::main]
async fn main() {
    // Parse the arguments and get the settings struct
    let settings = match cli::parse_args() {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            // Arguments unclear; simply exit
            std::process::exit(1);
        }
    };

    //let file_path = "websites.json"; // Name of the website list
    match parse_website_list(&settings.filepath_list).await {
        Ok(websites) => {
            // Verify websites
            if !settings.skip_verify {
                match website::verify_websites(&websites) {
                    Ok(_) => {
                        if settings.verbose {
                            println!("All websites verified.");
                        }
                    }
                    Err(err) => eprintln!("Verification error: {}", err),
                }
            }

            if !settings.dry_run {
                match html::generate_websites_html(&websites, &settings.filepath_template_redirect, &settings.filepath_template_index, settings.verbose).await {
                    Ok(_) => {
                        if settings.verbose {
                            println!("Finished generating webring HTML.");
                        }
                    }
                    Err(err) => eprintln!("Generation error: {}", err),
                }
            }
        }
        Err(err) => eprintln!("Error parsing website list: {} - ", err),
    }

    // Finally, copy files from /assets/ into the output folder 
    match file::copy_asset_files() {
        Ok(_) => {
            if settings.verbose {
                println!("Copied asset file(s) to webring folder");
            }
        }
        Err(err) => eprintln!("Error copying asset file(s): {}", err),
    }
}
