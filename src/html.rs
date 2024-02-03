use minify_html::{minify, Cfg};
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::file::acquire_file_data;
use crate::website::Website;

pub async fn generate_websites_html(
    websites: &[Website],
    path_output: &str,
    path_template_redirects: &str,
    path_template_index: &str,
    skip_minify: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load the redirect template
    let template_redirect = acquire_file_data(path_template_redirects).await?;

    // For minify
    let mut cfg = Cfg::new();
    cfg.keep_comments = true;

    // Generate HTML for each website
    for website in websites {
        match generate_html(websites, website, &path_output, &template_redirect, &cfg, skip_minify) {
            Ok(_) => {
                if verbose {
                    println!("Generated HTML for {}", website.url);
                }
            }
            Err(err) => eprintln!("Error generating for: {} - ", err),
        }
    }

    // Then generate the index/list page
    let template_index = acquire_file_data(path_template_index).await?;

    // Generate list table with replacements
    let replaced_content = template_index.replace(
        "<!-- TABLE_OF_WEBSITES -->",
        &generate_sites_table(websites)?,
    );

    // Create the list HTML
    let mut file = fs::File::create(format!("{}/list.html", path_output))?;

    // Minify the content
    if skip_minify
    {
        file.write_all(&replaced_content.as_bytes())?;
    } else {
        let minified_content = minify(replaced_content.as_bytes(), &cfg);
        file.write_all(&minified_content)?;
    }

    if verbose {
        println!("Generated list.html");
    }

    Ok(())
}

fn generate_html(
    websites: &[Website],
    website: &Website,
    path_output: &str,
    template_redirect: &str,
    minify_cfg: &Cfg,
    skip_minify: bool,
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

    let directory_path = format!("{}/{}", path_output, website.name);
    fs::create_dir_all(&directory_path)?;

    let next_html_path = Path::new(&directory_path).join("next.html");
    let previous_html_path = Path::new(&directory_path).join("previous.html");
    // TODO: Handle if there is no template
    let next_html_content = template_redirect.replace(
        "<!-- REDIRECT -->",
        &format!(
            "<meta http-equiv=\"refresh\" content=\"0; url={}\">",
            websites[next_index].url
        ),
    );
    let previous_html_content = template_redirect.replace(
        "<!-- REDIRECT -->",
        &format!(
            "<meta http-equiv=\"refresh\" content=\"0; url={}\">",
            websites[previous_index].url
        ),
    );

    // Minification before writing to file
    if skip_minify
    {
        fs::write(&next_html_path, next_html_content)?;
        fs::write(&previous_html_path, previous_html_content)?;
    } else {
        fs::write(
            &next_html_path,
            minify(&next_html_content.as_bytes(), &minify_cfg),
        )?;
        fs::write(
            &previous_html_path,
            minify(&previous_html_content.as_bytes(), &minify_cfg),
        )?;
    }

    Ok(())
}

pub fn generate_sites_table(websites: &[Website]) -> Result<String, Box<dyn std::error::Error>> {
    let mut table_html = String::new();

    // Open table tag
    table_html.push_str("<table>\n");

    // Table header
    table_html.push_str("    <thead>\n");
    table_html.push_str("        <tr>\n");
    table_html.push_str("            <th scope=\"col\">#</th>\n");
    table_html.push_str("            <th scope=\"col\">Name</th>\n");
    table_html.push_str("            <th scope=\"col\">URL</th>\n");
    table_html.push_str("            <th scope=\"col\">About</th>\n");
    table_html.push_str("            <th scope=\"col\">Owner</th>\n");
    table_html.push_str("        </tr>\n");
    table_html.push_str("    </thead>\n");

    // Table body
    table_html.push_str("    <tbody>\n");
    for (index, website) in websites.iter().enumerate() {
        table_html.push_str("        <tr>\n");

        table_html.push_str(&format!("            <td>{}</td>\n", index + 1));

        table_html.push_str("            <td>");
        table_html.push_str(&website.name);
        table_html.push_str("</td>\n");

        table_html.push_str("            <td><a href=\"");
        table_html.push_str(&website.url);
        table_html.push_str("\" target=\"_blank\">"); // target="_blank" to open in a new tab
        table_html.push_str(&website.url);
        table_html.push_str("</a></td>\n");

        table_html.push_str("            <td>");
        table_html.push_str(&website.about);
        table_html.push_str("</td>\n");

        table_html.push_str("            <td>");
        table_html.push_str(&website.owner);
        table_html.push_str("</td>\n");
        table_html.push_str("        </tr>\n");
    }
    table_html.push_str("    </tbody>\n");

    // Close table tag
    table_html.push_str("</table>\n");

    Ok(table_html)
}
