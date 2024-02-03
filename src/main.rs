mod cli;
mod file;
mod html;
mod http;
mod website;

#[tokio::main]
async fn main() {
    // Parse the arguments and get the settings struct
    let settings = cli::parse_args().await;

    match website::parse_website_list(&settings.filepath_list).await {
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
                let html_generator = html::HtmlGenerator::new(settings.skip_minify, settings.verbose);

                // Use the html_generator instance to generate websites html
                match html_generator.generate_websites_html(
                    &websites, 
                    &settings.path_output, 
                    &settings.filepath_template_redirect, 
                    &settings.filepath_template_index
                ).await {
                    Ok(_) => {
                        if settings.verbose {
                            println!("Finished generating webring HTML.");
                        }
                    },
                    Err(err) => eprintln!("Generation error: {}", err),
                }
            }
        }
        Err(err) => eprintln!("Error parsing website list: {} - ", err),
    }

    // Finally, copy files from /assets/ into the output folder 
    match file::copy_asset_files(&settings.path_assets, &settings.path_output) {
        Ok(_) => {
            if settings.verbose {
                println!("Copied asset file(s) to webring folder");
            }
        }
        Err(err) => eprintln!("Error copying asset file(s): {}", err),
    }
}
