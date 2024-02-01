use std::fs;
use std::path::Path;
use std::io::{BufReader, Write};

use crate::website::Website;

pub fn generate_html_files(websites: &[Website], website: &Website) -> Result<(), Box<dyn std::error::Error>> {
    let index = websites.iter().position(|w| w.name == website.name).unwrap();
    let previous_index = if index == 0 { websites.len() - 1 } else { index - 1 };
    let next_index = if index == websites.len() - 1 { 0 } else { index + 1 };

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



pub fn generate_list_html(websites: &[Website]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create("webring/list.html")?;

    write!(
        file,
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Webring List</title>
        </head>
        <body>
            <h1>Webring List</h1>
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>URL</th>
                    </tr>
                </thead>
                <tbody>
    "#
    )?;

    for website in websites {
        write!(
            file,
            r#"
                    <tr>
                        <td>{}</td>
                        <td><a href="{}">{}</a></td>
                    </tr>
            "#,
            website.name, website.url, website.url
        )?;
    }

    write!(
        file,
        r#"
                </tbody>
            </table>
        </body>
        </html>
    "#
    )?;

    Ok(())
}