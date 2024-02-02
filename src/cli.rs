use clap::{Arg, ArgAction, Command};

pub struct RingSettings {
    pub filepath_list: String,              // list of websites (JSON)
    pub filepath_template_redirect: String, // template for each redirect (HTML)
    pub filepath_template_index: String,    // template for the main list page (HTML)
    pub verbose: bool,                      // whether to output console messages
    pub skip_verify: bool,                  // not recommended lol
    pub dry_run: bool,                      // runs without outputting files
}

pub fn parse_args() -> Result<RingSettings, &'static str> {
    // Setting up command line argument parsing
    let matches = Command::new("rustring")
        .version("0.1.0")
        .author("Kern AKA Kersed")
        .about("Generates HTML files to create a static webring system.")
        .arg(
            Arg::new("list") // Specify the input list file containing the ring's websites
                .short('l') // Short flag -l
                .long("list") // Long flag --list
                .ignore_case(true)
                .value_name("FILE") // Name of the value in help messages
                .default_value("./websites.json")
                .help("Sets the input list file to use"),
        )
        .arg(
            Arg::new("path-template-redirect")
                .short('r')
                .long("path-template-redirect")
                .ignore_case(true)
                .value_name("FILE") // Name of the value in help messages
                .default_value("./templates/redirect_template.html")
                .help("Define the HTML template used to generate each website's redirect pages"),
        )
        .arg(
            Arg::new("path-template-index")
                .short('i')
                .long("path-template-index")
                .ignore_case(true)
                .value_name("FILE") // Name of the value in help messages
                .default_value("./templates/list_template.html")
                .help("Define the HTML template used to generate the main list/index page"),
        )
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

    // Check string inputs 
    // These should all have default values
    let filepath_list = get_arg(&matches, "list")?;
    let filepath_template_redirect = get_arg(&matches, "path-template-redirect")?;
    let filepath_template_index = get_arg(&matches, "path-template-index")?;

    // Construct settings struct
    Ok(RingSettings {
        filepath_list,
        filepath_template_redirect,
        filepath_template_index,
        verbose,
        skip_verify,
        dry_run,
    })
}

fn get_arg(matches: &clap::ArgMatches, arg_name: &str) -> Result<String, &'static str> {
    if let Some(value) = matches.get_one::<String>(arg_name) {
        // Not sure whether to log these, leaning against it
        /*
        if verbose {
            println!("Using {}: {}", arg_name, value);
        }
        */
        Ok(value.clone())
    } else {
        // Handle the error case, given detailed feedback on which argument was invalid
        eprintln!("Error: invalid or missing value for {}!", arg_name);
        Err("Invalid or missing argument provided.")
    }
}
