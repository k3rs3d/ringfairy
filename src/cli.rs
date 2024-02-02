use clap::{Arg, ArgAction, Command};

pub struct RingSettings {
    pub list_filepath: String,
    pub verbose: bool,
    pub skip_verify: bool,
    pub dry_run: bool,
}

pub fn parse_args() -> Result<RingSettings, &'static str> {
    // Setting up command line argument parsing
    let matches = Command::new("rustring")
        .version("0.1.0")
        .author("Kern AKA Kersed")
        .about("Generates HTML files to create a static webring system.")
        // Define the -f/--file argument
        .arg(
            Arg::new("list") // Specify the input list file containing the ring's websites
                .short('l') // Short flag -f
                .long("list") // Long flag --file
                .ignore_case(true)
                .value_name("FILE") // Name of the value in help messages
                .default_value("./websites.json")
                .help("Sets the input list file to use"),
        ) // Help message
        .arg(
            Arg::new("verbose") // Enable verbose logging mode (print more console messages)
                .short('v') // Short flag
                .long("verbose") // Long flag
                .action(ArgAction::SetTrue) // Makes it a flag without a value
                .help("Enables verbose logging"),
        )
        .arg(
            Arg::new("skip-verification") // Skip some error-checking when building webring
                .long("skip-verification")
                .action(ArgAction::SetTrue)
                .help("Skips verification of the URLs in the list"),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .help("Perform a dry run without writing any files"),
        )
        .get_matches();

    // Check toggle flags
    // e.g. returns true if -v/--verbose was used, otherwise false:
    let verbose = *matches.get_one::<bool>("verbose").unwrap_or(&false);
    let dry_run = *matches.get_one::<bool>("dry-run").unwrap_or(&false);
    let skip_verify = *matches
        .get_one::<bool>("skip-verification")
        .unwrap_or(&false);

    let file_path_option = matches.get_one::<String>("list").cloned();

    // Because of default_value, file_path_option should always contain Some
    // However, generalizing:
    let list_filepath = if let Some(path) = file_path_option {
        if verbose {
            println!("Using website list: {}", path);
        }
        path
    } else {
        // Handle the error case, if it's possible for this to be None (e.g., for other arguments without default_value)
        eprintln!("Error: invalid website list!");
        // Directly return an error for now
        return Err("Invalid website list provided.");
    };

    // Construct settings struct
    Ok(RingSettings {
        list_filepath,
        verbose,
        skip_verify,
        dry_run,
    })
}
