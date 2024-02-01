use clap::{Arg, Command};

pub struct RingSettings
{
    pub list_filepath: String,
}

pub fn parse_args() -> Result<RingSettings, &'static str> {
            // Setting up command line argument parsing
            let matches = Command::new("rustring")
            .version("0.1.0")
            .author("Kern AKA Kersed")
            .about("Generates HTML files to create a static webring system")
            // Define the -f/--file argument
            .arg(Arg::new("list")
                .short('l') // Short flag -f
                .long("list") // Long flag --file
                .ignore_case(true)
                //.takes_value(true) // This flag takes a value
                .value_name("FILE") // Name of the value in help messages
                .default_value("./websites.json")
                .help("Sets the input file to use")) // Help message
            .get_matches();

        
            let file_path_option = matches.get_one::<String>("list").cloned();

            // In this specific case, because of default_value, file_path_option should always contain Some
            // However, generalizing the approach for other arguments:
            let file_path = if let Some(path) = file_path_option {
                println!("Using website list: {}", path);
                path
            } else {
                // Handle the error case, if it's possible for this to be None (e.g., for other arguments without default_value)
                eprintln!("Error: invalid website list!");
                // For this tutorial example, we will directly return an error, but you might handle it differently.
                return Err("Invalid website list provided.");
            };
        
            // Now that we have file_path, we can construct the settings struct
            Ok(RingSettings {
                list_filepath: file_path,
            })
}