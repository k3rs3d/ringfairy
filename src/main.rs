mod cli;
mod file;
mod html;
mod http;
mod website;

#[tokio::main]
async fn main() {
    // Parse the arguments and get settings struct
    let settings = cli::parse_args().await;

    // Init logging
    env_logger::init();
    log::debug!("Initialized logging.");
    log::trace!("Parsed arguments.");

    // Perform webring generation
    if let Err(e) = website::process_websites(&settings).await {
        log::error!("Process error: {}", e);
        return;
    }

    // Finally, copy files from ./assets (by default) into the output folder
    if let Err(e) = file::copy_asset_files(&settings.path_assets, &settings.path_output) {
        log::error!("Error copying asset file(s): {}", e);
    }

    log::info!("Done!");
}
