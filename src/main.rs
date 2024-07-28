mod cli;
mod error;
mod file;
mod gen;
mod http;
mod website;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    // Parse arguments & get settings struct
    let settings = cli::parse_args().await;

    // Init logging
    env_logger::init();
    log::info!("Starting with settings: {:?}", settings);

    // Start a timer
    let start = std::time::Instant::now();

    // Generate webring
    gen::make_ringfairy_go_now(&settings).await?;

    // Calculate elapsed time
    let elapsed = start.elapsed();
    log::info!("Done in {} ms!", elapsed.as_millis());

    Ok(())
}
