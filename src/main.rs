mod cli;
mod error;
mod file;
mod format_errors;
mod gen;
mod http;
mod website;

async fn run_ringfairy() -> Result<(), error::Error> {
    // Parse arguments & get settings struct
    let settings = cli::parse_args().await?;

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

#[tokio::main]
async fn main() {
    if let Err(err) = run_ringfairy().await {
        println!("{}", format_errors::format_error(err));
        std::process::exit(1);
    };
}
