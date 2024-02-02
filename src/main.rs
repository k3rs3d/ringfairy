use std::fs;
use std::io::BufReader;

mod cli;
mod html;
mod website;

use crate::html::*;
use crate::website::Website;

// Load the websites from JSON
fn parse_website_list(file_path: &str) -> Result<Vec<Website>, Box<dyn std::error::Error>> {
    // Load JSON file
    let file = fs::File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    // Parse JSON contents
    let reader = BufReader::new(file);
    let websites: Vec<Website> =
        serde_json::from_reader(reader).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(websites)
}

fn copy_template_files() -> Result<(), Box<dyn std::error::Error>> {
    // Create webring directory (if it doesn't exist)
    fs::create_dir_all("webring")
        .map_err(|e| format!("Failed to create webring directory: {}", e))?;

    // (try to) Copy styles.css to webring folder
    fs::copy("templates/styles.css", "webring/styles.css")
        .map_err(|e| format!("Failed to copy styles.css: {}", e))?;

    Ok(())
}

fn main() {
    // Parse the arguments and get the settings struct
    let settings = match cli::parse_args() {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            // Arguments unclear; simply exit
            std::process::exit(1);
        }
    };

    // Currently just used for `styles.css` I think
    match copy_template_files() {
        Ok(_) => println!("Copied template(s) to webring folder"),
        Err(err) => eprintln!("Error copying templates: {}", err),
    }

    //let file_path = "websites.json"; // Name of the website list
    match parse_website_list(&settings.list_filepath) {
        Ok(websites) => {
            // Website verification
            if !settings.skip_verify {
                match website::verify_websites(&websites) {
                    Ok(_) => {
                        if settings.verbose {
                            println!("All websites verified successfully.");
                        }
                    }
                    Err(err) => eprintln!("Verification error: {}", err),
                }
            } else if settings.verbose {
                println!("Skipping website verification.");
            }

            // Generate folder + HTML files for each website in the list
            for website in &websites {
                match generate_html_files(&websites, website) {
                    Ok(_) => println!("Generated HTML for {}", website.url),
                    Err(err) => eprintln!("Error generating for: {} - ", err),
                }
            }

            // Create the main list/index page
            match generate_list_html(&websites) {
                Ok(_) => println!("Generated list.html"),
                Err(err) => eprintln!("Error generating list.html: {} - ", err),
            }
        }
        Err(err) => eprintln!("Error parsing website list: {} - ", err),
    }
}
