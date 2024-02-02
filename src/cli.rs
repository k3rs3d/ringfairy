use clap::{Parser, ArgAction};

#[derive(Parser, Debug)]
#[clap(name = "rustring", version = "0.1.0", author = "Kern AKA Kersed", about = "Generates HTML files to create a static webring system.")]
pub struct RingSettings {
    #[clap(short = 'l', long, ignore_case = true, default_value_t = String::from("./websites.json"), help = "Specify the file containing the list of websites to use as input")]
    pub filepath_list: String,

    #[clap(short = 'o', long = "path-output", ignore_case = false, default_value_t = String::from("./webring"), help = "Define the directory where the generated files will be saved.")]
    pub path_output: String,

    #[clap(short = 'a', long = "path-assets", ignore_case = false, default_value_t = String::from("./assets"), help = "Specify the directory where asset files (e.g. CSS, images, other extras) can be found. NOTE: All contents will be copied into the output directory!")]
    pub path_assets: String,

    #[clap(short = 'r', long = "path-template-redirect", ignore_case = false, default_value_t = String::from("./templates/redirect_template.html"), help = "Specify the HTML template used to generate each website's redirect pages. It should contain '<!-- REDIRECT -->' somewhere in the file.")]
    pub filepath_template_redirect: String,

    #[clap(short = 'i', long = "path-template-index", ignore_case = false, default_value_t = String::from("./templates/list_template.html"), help = "Specify the HTML template used to generate the main list page. It should contain '<!-- TABLE_OF_WEBSITES -->' somewhere in the file.")]
    pub filepath_template_index: String,
    
    #[clap(short = 'v', long, action = ArgAction::SetTrue, help = "Enables verbose logging")]
    pub verbose: bool,
    
    #[clap(long = "skip-verification", action = ArgAction::SetTrue, help = "Skips verification of the URLs in the list. Probably unwise!")]
    pub skip_verify: bool,
    
    #[clap(long = "dry-run", action = ArgAction::SetTrue, help = "Perform a dry run without writing any files")]
    pub dry_run: bool,
}

// Simplified argument parsing
pub fn parse_args() -> RingSettings {
    RingSettings::parse()
}