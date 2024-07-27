mod cli;
mod error;
mod file;
mod html;
mod http;
mod webring;
mod website;

#[tokio::main]
async fn main() {
    // Init logging
    env_logger::init();

    // Parse arguments & get settings struct
    let settings = cli::parse_args().await;
    log::info!("Starting generator with settings: {:?}", settings);

    // Start a timer 
    let start = std::time::Instant::now();

    // Generate webring 
    if let Err(e) = website::process_websites(&settings).await {
        log::error!("Process error: {}", e);
        return;
    }

    // Finally, copy static files (from ./assets by default) into output folder
    if let Err(e) = file::copy_asset_files(&settings.path_assets, &settings.path_output) {
        log::error!("Error copying asset file(s): {}", e);
    }

    // Calculate elapsed time
    let elapsed = start.elapsed();
    log::info!("Done in {} ms!", elapsed.as_millis());
}
