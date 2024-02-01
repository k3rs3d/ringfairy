use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::io::{BufReader, Write};

mod html;
mod website;

use crate::website::Website;
use crate::html::*;

fn parse_website_list(file_path: &str) -> Result<Vec<Website>, Box<dyn std::error::Error>> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let websites: Vec<Website> = serde_json::from_reader(reader)?;
    Ok(websites)
}

fn copy_template_files() -> Result<(), Box<dyn std::error::Error>> {
    // Create webring directory (if it doesn't exist)
    fs::create_dir_all("webring")?;

    // Copy styles.css to webring folder
    fs::copy("templates/styles.css", "webring/styles.css")?;

    Ok(())
}


fn main() {
    match copy_template_files() {
        Ok(_) => println!("Copied CSS template(s) to webring folder"),
        Err(err) => eprintln!("Error copying: {}", err),
    }

    let file_path = "websites.json";
    match parse_website_list(file_path) {
        Ok(websites) => {
            for website in &websites {
                match generate_html_files(&websites, website) {
                    Ok(_) => println!("Generated HTML files for {}", website.url),
                    Err(err) => eprintln!("Error generating HTML files: {}", err),
                }
            }

            // Create the main list 
            match generate_list_html(&websites) {
                Ok(_) => println!("Generated list.html"),
                Err(err) => eprintln!("Error generating list.html: {}", err),
            }
        }
        Err(err) => eprintln!("Error parsing website list: {}", err),
    }
}
