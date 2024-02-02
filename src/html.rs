use std::fs;
use std::io::Write;
use std::path::Path;

use crate::website::Website;

pub fn generate_index_html(
    websites: &[Website],
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load the list template
    let template = fs::read_to_string("templates/list_template.html")?;

    // Create the list HTML
    let mut file = fs::File::create("webring/list.html")?;

    write!(
        file,
        "{}",
        template.replace(
            "<!-- TABLE_OF_WEBSITES -->",
            &generate_sites_table(websites)?
        )
    )?;

    if verbose {
        println!("Generated list.html");
    }

    Ok(())
}

pub fn generate_websites_html(websites: &[Website], verbose: bool) {
    for website in websites {
        match generate_html(websites, website) {
            Ok(_) => {
                if verbose {
                    println!("Generated HTML for {}", website.url);
                }
            }
            Err(err) => eprintln!("Error generating for: {} - ", err),
        }
    }
}

fn generate_html(
    websites: &[Website],
    website: &Website,
) -> Result<(), Box<dyn std::error::Error>> {
    let index = websites
        .iter()
        .position(|w| w.name == website.name)
        .unwrap();
    let previous_index = if index == 0 {
        websites.len() - 1
    } else {
        index - 1
    };
    let next_index = if index == websites.len() - 1 {
        0
    } else {
        index + 1
    };

    let directory_path = format!("webring/{}", website.name);
    fs::create_dir_all(&directory_path)?;

    let next_html_path = Path::new(&directory_path).join("next.html");
    let previous_html_path = Path::new(&directory_path).join("previous.html");

    let next_html_content = format!("<link rel=\"stylesheet\" href=\"../styles.css\"><meta http-equiv=\"refresh\" content=\"0; url={}\">", websites[next_index].url);
    let previous_html_content = format!("<link rel=\"stylesheet\" href=\"../styles.css\"><meta http-equiv=\"refresh\" content=\"0; url={}\">", websites[previous_index].url);

    fs::write(&next_html_path, next_html_content)?;
    fs::write(&previous_html_path, previous_html_content)?;

    Ok(())
}

pub fn generate_sites_table(websites: &[Website]) -> Result<String, Box<dyn std::error::Error>> {
    let mut table_html = String::new();

    // Open table tag
    table_html.push_str("<table>\n");

    // Table header
    table_html.push_str("    <thead>\n");
    table_html.push_str("        <tr>\n");
    table_html.push_str("            <th>Name</th>\n");
    table_html.push_str("            <th>URL</th>\n");
    table_html.push_str("        </tr>\n");
    table_html.push_str("    </thead>\n");

    // Table body
    table_html.push_str("    <tbody>\n");
    for website in websites {
        table_html.push_str("        <tr>\n");
        table_html.push_str("            <td>");
        table_html.push_str(&website.name);
        table_html.push_str("</td>\n");
        table_html.push_str("            <td><a href=\"");
        table_html.push_str(&website.url);
        table_html.push_str("\">");
        table_html.push_str(&website.url);
        table_html.push_str("</a></td>\n");
        table_html.push_str("        </tr>\n");
    }
    table_html.push_str("    </tbody>\n");

    // Close table tag
    table_html.push_str("</table>\n");

    Ok(table_html)
}
