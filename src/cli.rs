use clap::{ArgAction, Parser};
use serde::Deserialize;

use crate::file;

// Main/final settings struct
#[derive(Debug)]
pub struct AppSettings {
    pub ring_name: String,
    pub ring_description: String,
    pub ring_owner: String,
    pub ring_owner_site: String,
    pub filepath_config: String,
    pub filepath_list: String,
    pub path_output: String,
    pub path_assets: String,
    pub path_templates: String,
    pub base_url: String,
    pub client_user_agent: String,
    pub client_header: String,
    pub audit_retries_max: u8,
    pub audit_retries_delay: u8,
    pub audit: bool,
    pub no_slug: bool,
    pub shuffle: bool,
    pub verbose: bool,
    pub skip_minify: bool,
    pub skip_verify: bool,
    pub dry_run: bool,
}

// Hardcoded values for anything not defined elsewhere
impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            ring_name: "webring".into(),
            ring_description: "A ring that connects websites to each other with links".into(),
            ring_owner: "Webring Organization or Person".into(),
            ring_owner_site: "https://webring.domain.tld/".into(),
            filepath_config: "./config.json".into(),
            filepath_list: "./websites.json".into(),
            path_output: "./webring".into(),
            path_assets: "./data/assets".into(),
            path_templates: "./data/templates".into(),
            base_url: " ".to_string(),
            client_user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.212 Safari/537.36".into(),
            client_header: "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".into(),
            audit_retries_delay: 100,
            audit_retries_max: 2,
            audit: false,
            no_slug: false,
            shuffle: false,
            verbose: false,
            skip_minify: false,
            skip_verify: false,
            dry_run: false,
        }
    }
}

// Config settings loaded from config file, derive Default
#[derive(Deserialize, Debug, Default)]
pub struct ConfigSettings {
    pub ring_name: Option<String>,
    pub ring_description: Option<String>,
    pub ring_owner: Option<String>,
    pub ring_owner_site: Option<String>,
    pub filepath_list: Option<String>,
    pub path_output: Option<String>,
    pub path_assets: Option<String>,
    pub path_templates: Option<String>,
    pub base_url: Option<String>,
    pub client_user_agent: Option<String>,
    pub client_header: Option<String>,
    pub audit_retries_max: Option<u8>,
    pub audit_retries_delay: Option<u8>,
    pub audit: Option<bool>,
    pub no_slug: Option<bool>,
    pub shuffle: Option<bool>,
    pub verbose: Option<bool>,
    pub skip_minify: Option<bool>,
    pub skip_verify: Option<bool>,
    pub dry_run: Option<bool>,
}

// Clap-specific settings struct - able to contain Options
#[derive(Parser, Debug)]
#[clap(
    name = "ringfairy",
    version = env!("CARGO_PKG_VERSION"),
    author = "Kern AKA Kersed",
    about = "Creates a webring by generating HTML files for a set of websites, linking them together."
)]
pub struct ClapSettings {
    #[clap(
        short = 'c',
        long = "cfg",
        ignore_case = false,
        default_value = "./config.json",
        help = "Specify the config file path. Useful for settings that stay constant across many runs of your application, like path locations. Remember, any settings specified via command-line arguments will override the corresponding ones from this file"
    )]
    pub filepath_config: Option<String>,

    #[clap(
        short = 'l',
        long = "list",
        ignore_case = false,
        help = "Specify the file containing the list of websites to use. It should be a JSON file with 'name', 'url', etc fields."
    )]
    pub filepath_list: Option<String>,

    #[clap(
        short = 'o',
        long = "output",
        ignore_case = false,
        help = "Define the output directory. Generated files will be saved in this folder."
    )]
    pub path_output: Option<String>,

    #[clap(
        short = 'a',
        long = "assets",
        ignore_case = false,
        help = "Specify the directory where asset files (e.g. CSS, images, other extras) can be found. NOTE: All contents will be copied into the output directory!"
    )]
    pub path_assets: Option<String>,

    #[clap(
        short = 't',
        long = "templates",
        ignore_case = false,
        help = "Specify the folder containing HTML templates to use. Should at least contain 'templates.html' for creating the 'next' and 'previous' pages."
    )]
    pub path_templates: Option<String>,

    #[clap(
        short = 'u',
        long = "url",
        ignore_case = false,
        help = "The base URL for the webring. Something like 'https://example.com'"
    )]
    pub base_url: Option<String>,

    #[clap(
        short = 'n',
        long = "name",
        ignore_case = false,
        help = "The name of the webring. Something like 'Ghostring'."
    )]
    pub ring_name: Option<String>,

    #[clap(
        short = 'd',
        long = "description",
        ignore_case = false,
        help = "A short description/about the webring."
    )]
    pub ring_description: Option<String>,

    #[clap(
        short = 'm',
        long = "maintainer",
        ignore_case = false,
        help = "The owner/maintainer of the webring, could be a person or an organization."
    )]
    pub ring_owner: Option<String>,

    #[clap(
        short = 'w',
        long = "website",
        ignore_case = false,
        help = "The website link of the website owner, not the base URL of the webring."
    )]
    pub ring_owner_site: Option<String>,

    #[clap(short = 'A', long = "audit", action = ArgAction::SetTrue, help = "Scrapes URLs to check for the webring links before adding them to the list. If the links can't be found, the site will get skipped. ")]
    pub audit: bool,

    #[clap(short = 'M', long = "retries-max", help = "When auditing sites, how many times to retry connecting to a site before giving up. ")]
    pub audit_retries_max: Option<u8>,

    #[clap(short = 'D', long = "retries-delay", help = "When auditing sites, how many miliseconds to wait before trying again. ")]
    pub audit_retries_delay: Option<u8>,

    #[clap(short = 'U', long = "client-user-agent", help = "When auditing sites, user-agent string for the scraper. ")]
    pub client_user_agent: Option<String>,

    #[clap(short = 'H', long = "client-header", help = "When auditing sites, header string for the scraper. ")]
    pub client_header: Option<String>,

    #[clap(short = 's', long = "shuffle", action = ArgAction::SetTrue, help = "Randomly shuffles the website sequence when generating the webring (does not modify the website list file).")]
    pub shuffle: bool,

    #[clap(short = 'v', long = "verbose", action = ArgAction::Count, help = "Enables verbose logging. Set -vv for very verbose.")]
    pub verbose: u8,

    #[clap(long = "no-slug", action = ArgAction::SetTrue, help = "Makes the webring reference sites by their index, rather than their slug. So the first website would be under /1/, the second /2/, etc. The default behavior is to create directories named for the site slug. ")]
    pub no_slug: bool,

    #[clap(long = "skip-minification", action = ArgAction::SetTrue, help = "Skips 'minification' of HTML files, which tries to reduce their file size. If your generated HTML files are having issues, try skipping minification.")]
    pub skip_minify: bool,

    #[clap(long = "skip-verification", action = ArgAction::SetTrue, help = "Skips verification of the URLs in the list. Probably unwise!")]
    pub skip_verify: bool,

    #[clap(long = "dry-run", action = ArgAction::SetTrue, help = "Perform a dry run without writing any files.")]
    pub dry_run: bool,
}

async fn load_config(config_path: &str) -> Option<ConfigSettings> {
    // Early return for an empty path
    if config_path.trim().is_empty() {
        return None;
    }

    // Load async (supports either locally or remotely)
    let config_content = match file::acquire_file_data(config_path).await {
        Ok(content) => content,
        Err(_) => return None,
    };

    // Ensure the file is not empty
    if config_content.trim().is_empty() {
        return None;
    }

    // Deserialize based on format
    // TODO: Add more config file types?
    match file::get_extension_from_path(config_path).as_deref() {
        Some("json") => match serde_json::from_str(&config_content) {
            Ok(config) => Some(config),
            Err(_) => None, // JSON deserialization failed
        },
        Some("toml") => match toml::from_str(&config_content) {
            Ok(config) => Some(config),
            Err(_) => None, // TOML deserialization failed
        },
        _ => None, // Should not reach here
    }
}

fn merge_configs(cli_args: ClapSettings, config: self::ConfigSettings) -> AppSettings {
    let mut final_settings = AppSettings::default();

    final_settings.ring_name = cli_args
        .ring_name
        .or(config.ring_name)
        .unwrap_or(final_settings.ring_name);
    final_settings.ring_description = cli_args
        .ring_description
        .or(config.ring_description)
        .unwrap_or(final_settings.ring_description);
    final_settings.ring_owner = cli_args
        .ring_owner
        .or(config.ring_owner)
        .unwrap_or(final_settings.ring_owner);
    final_settings.ring_owner_site = cli_args
        .ring_owner_site
        .or(config.ring_owner_site)
        .unwrap_or(final_settings.ring_owner_site);
    final_settings.filepath_list = cli_args
        .filepath_list
        .or(config.filepath_list)
        .unwrap_or(final_settings.filepath_list);
    final_settings.path_output = cli_args
        .path_output
        .or(config.path_output)
        .unwrap_or(final_settings.path_output);
    final_settings.path_assets = cli_args
        .path_assets
        .or(config.path_assets)
        .unwrap_or(final_settings.path_assets);
    final_settings.path_templates = cli_args
        .path_templates
        .or(config.path_templates)
        .unwrap_or(final_settings.path_templates);
    final_settings.base_url = cli_args
        .base_url
        .or(config.base_url)
        .unwrap_or(final_settings.base_url);

    final_settings.client_header = cli_args.client_header.or(config.client_header).unwrap_or(final_settings.client_header);
    final_settings.client_user_agent = cli_args.client_user_agent.or(config.client_user_agent).unwrap_or(final_settings.client_user_agent);

    final_settings.audit_retries_delay = cli_args.audit_retries_delay.or(config.audit_retries_delay).unwrap_or(final_settings.audit_retries_delay);
    final_settings.audit_retries_max = cli_args.audit_retries_max.or(config.audit_retries_max).unwrap_or(final_settings.audit_retries_max);

    final_settings.audit = cli_args.audit || config.audit.unwrap_or(final_settings.audit);
    final_settings.no_slug = cli_args.no_slug || config.no_slug.unwrap_or(final_settings.no_slug);
    final_settings.shuffle = cli_args.shuffle || config.shuffle.unwrap_or(final_settings.shuffle);
    //final_settings.verbose = cli_args.verbose || config.verbose.unwrap_or(final_settings.verbose);
    final_settings.skip_minify =
        cli_args.skip_minify || config.skip_minify.unwrap_or(final_settings.skip_minify);
    final_settings.skip_verify =
        cli_args.skip_verify || config.skip_verify.unwrap_or(final_settings.skip_verify);
    final_settings.dry_run = cli_args.dry_run || config.dry_run.unwrap_or(final_settings.dry_run);

    // HACK: just set the config file value, then CLI value, directly
    std::env::set_var("RUST_LOG", "error"); // Default to only showing errors

    if config.verbose.unwrap() {
        std::env::set_var("RUST_LOG", "warn");
    }
    // HACK ish: apply log level settings here
    match cli_args.verbose {
        0 => (),
        1 => std::env::set_var("RUST_LOG", "warn"), // Showing info level logs with -v
        2 => std::env::set_var("RUST_LOG", "info"), // Showing info level logs with -vv
        _ => std::env::set_var("RUST_LOG", "debug"), // Showing debug logs with -vvv or more
    }

    final_settings
}

pub async fn parse_args() -> AppSettings {
    let clap_args = ClapSettings::parse();

    // Check if a config file path is provided, and it's not empty
    let config_args = load_config(clap_args.filepath_config.as_deref().unwrap_or(""))
        .await
        .unwrap_or_default();

    merge_configs(clap_args, config_args)
}
