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


fn main() {
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
