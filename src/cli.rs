use clap::{Parser, ArgAction};
use serde::Deserialize;

use crate::file;

// Main/final settings struct
#[derive(Debug)]
pub struct AppSettings {
    pub filepath_list: String,
    pub path_output: String,
    pub path_assets: String,
    pub filepath_template_redirect: String,
    pub filepath_template_index: String,
    pub verbose: bool,
    pub skip_verify: bool,
    pub dry_run: bool,
}

// Hardcoded values for anything not defined elsewhere
impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            filepath_list: "./websites.json".into(),
            path_output: "./webring".into(),
            path_assets: "./assets".into(),
            filepath_template_redirect: "./templates/redirect_template.html".into(),
            filepath_template_index: "./templates/list_template.html".into(),
            verbose: false,
            skip_verify: false,
            dry_run: false,
        }
    }
}

// Clap-specific settings struct - able to contain Options
#[derive(Parser, Debug)]
#[clap(name = "rustring", version = "0.1.0", author = "Kern AKA Kersed", about = "Generates HTML files to create a static webring system.")]
pub struct ClapSettings {
    #[clap(short = 'c', long = "cfg", default_value("./config.json"), ignore_case = false, help = "Specify the config file path")]
    pub filepath_config: String,

    #[clap(short = 'l', long = "list", ignore_case = false, help = "Specify the file containing the list of websites to use as input")]
    pub filepath_list: Option<String>,

    #[clap(short = 'o', long = "path-output", ignore_case = false, help = "Define the directory where the generated files will be saved.")]
    pub path_output: Option<String>,

    #[clap(short = 'a', long = "path-assets", ignore_case = false, help = "Specify the directory where asset files (e.g. CSS, images, other extras) can be found. NOTE: All contents will be copied into the output directory!")]
    pub path_assets: Option<String>,

    #[clap(short = 'r', long = "path-template-redirect", ignore_case = false, help = "Specify the HTML template used to generate each website's redirect pages. It should contain '<!-- REDIRECT -->' somewhere in the file.")]
    pub filepath_template_redirect: Option<String>,

    #[clap(short = 'i', long = "path-template-index", ignore_case = false, help = "Specify the HTML template used to generate the main list page. It should contain '<!-- TABLE_OF_WEBSITES -->' somewhere in the file.")]
    pub filepath_template_index: Option<String>,
    
    #[clap(short = 'v', long, action = ArgAction::SetTrue, help = "Enables verbose logging")]
    pub verbose: bool,
    
    #[clap(long = "skip-verification", action = ArgAction::SetTrue, help = "Skips verification of the URLs in the list. Probably unwise!")]
    pub skip_verify: bool,
    
    #[clap(long = "dry-run", action = ArgAction::SetTrue, help = "Perform a dry run without writing any files")]
    pub dry_run: bool,
}

// Contains settings loaded from config file, e.g., config.json 
#[derive(Deserialize, Debug)]
pub struct ConfigSettings {
    pub filepath_list: Option<String>,
    pub path_output: Option<String>,
    pub path_assets: Option<String>,
    pub filepath_template_redirect: Option<String>,
    pub filepath_template_index: Option<String>,
    pub verbose: Option<bool>,
    pub skip_verify: Option<bool>,
    pub dry_run: Option<bool>,
}

async fn load_config(config_path: &str) -> Option<ConfigSettings> {
    let config_content = match file::acquire_file_data(config_path).await {
        Ok(content) => content,
        Err(_) => return None,
    };
    serde_json::from_str(&config_content).ok()
}

fn merge_configs(cli_args: ClapSettings, config: Option<ConfigSettings>) -> AppSettings {
    let mut final_settings = AppSettings::default();

    if let Some(conf) = config {
        // Apply settings from config.json where available (unwrap_or keeps original if None)
        final_settings.filepath_list = conf.filepath_list.unwrap_or(final_settings.filepath_list);
        final_settings.path_output = conf.path_output.unwrap_or(final_settings.path_output);
        final_settings.path_assets = conf.path_assets.unwrap_or(final_settings.path_assets);
        final_settings.filepath_template_redirect = conf.filepath_template_redirect.unwrap_or(final_settings.filepath_template_redirect);
        final_settings.filepath_template_index = conf.filepath_template_index.unwrap_or(final_settings.filepath_template_index);
        final_settings.verbose = conf.verbose.unwrap_or(final_settings.verbose);
        final_settings.skip_verify = conf.skip_verify.unwrap_or(final_settings.skip_verify);
        final_settings.dry_run = conf.dry_run.unwrap_or(final_settings.dry_run);
    }

    // Then, override with CLI arguments if provided
    if let Some(val) = cli_args.filepath_list { final_settings.filepath_list = val; }
    if let Some(val) = cli_args.path_output { final_settings.path_output = val; }
    if let Some(val) = cli_args.path_assets { final_settings.path_assets = val; }
    if let Some(val) = cli_args.filepath_template_redirect { final_settings.filepath_template_redirect = val; }
    if let Some(val) = cli_args.filepath_template_index { final_settings.filepath_template_index = val; }

    // Boolean flags can simply be overridden as they don't have a `None` state
    if cli_args.verbose { final_settings.verbose = cli_args.verbose; }
    if cli_args.skip_verify { final_settings.skip_verify = cli_args.skip_verify; }
    if cli_args.dry_run { final_settings.dry_run = cli_args.dry_run; }
    
    final_settings
}

pub async fn parse_args() -> AppSettings {
    let clap_args = ClapSettings::parse();
    let config_args = load_config(&clap_args.filepath_config).await;

    merge_configs(clap_args, config_args)
}