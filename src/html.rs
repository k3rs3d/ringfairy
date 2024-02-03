use minify_html::{minify, Cfg};
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::file::acquire_file_data;
use crate::website::Website;

pub struct HtmlGenerator {
    cfg: Cfg,
    skip_minify: bool,
}

impl HtmlGenerator {
    pub fn new(skip_minify: bool) -> Self {
        let mut cfg = Cfg::new();
        cfg.keep_comments = true;
        Self {
            cfg,
            skip_minify,
        }
    }

    async fn acquire_template(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        acquire_file_data(path).await
    }

    fn write_content(
        &self,
        file_path: &Path,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::create(file_path)?;
        let final_content = if self.skip_minify {
            content.to_string()
        } else {
            let minified = minify(content.as_bytes(), &self.cfg);
            String::from_utf8(minified)?
        };

        file.write_all(final_content.as_bytes())?;

        log::info!("Generated HTML file {}", file_path.display());

        Ok(())
    }

    pub async fn generate_websites_html(
        &self,
        websites: &[Website],
        path_output: &str,
        path_template_redirects: &str,
        path_template_index: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get template file for redirect pages
        log::debug!("Attempting to load webring redirect HTML template...");
        let template_redirect = self.acquire_template(path_template_redirects).await?;

        for (index, website) in websites.iter().enumerate() {
            // Determine index
            let prev_index = if index == 0 {
                websites.len() - 1
            } else {
                index - 1
            };
            let next_index = if index == websites.len() - 1 {
                0
            } else {
                index + 1
            };
            let previous_website = &websites[prev_index];
            let next_website = &websites[next_index];

            // Generate HTML for this website
            self.generate_html(website, previous_website, next_website, path_output, &template_redirect)?;
        }

        self.generate_list_html(websites, path_output, path_template_index)
            .await?;

        Ok(())
    }

    fn generate_html(
        &self,
        website: &Website,
        previous_website: &Website,
        next_website: &Website,
        path_output: &str,
        template_redirect: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let directory_path = format!("{}/{}", path_output, website.name);
        fs::create_dir_all(&directory_path)?;

        let next_html_path = Path::new(&directory_path).join("next.html");
        let previous_html_path = Path::new(&directory_path).join("previous.html");
        // TODO: Handle if there is no template
        let next_html_content = template_redirect.replace(
            "<!-- REDIRECT -->",
            &format!(
                "<meta http-equiv=\"refresh\" content=\"0; url={}\">",
                next_website.url
            ),
        );
        let previous_html_content = template_redirect.replace(
            "<!-- REDIRECT -->",
            &format!(
                "<meta http-equiv=\"refresh\" content=\"0; url={}\">",
                previous_website.url
            ),
        );

        self.write_content(&previous_html_path, &previous_html_content)?;
        self.write_content(&next_html_path, &next_html_content)?;

        Ok(())
    }

    async fn generate_list_html(
        &self,
        websites: &[Website],
        path_output: &str,
        path_template_index: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let template_index = self.acquire_template(path_template_index).await?;
        let replaced_content = template_index.replace(
            "<!-- TABLE_OF_WEBSITES -->",
            &self.generate_sites_table(websites)?,
        );

        let file_path = Path::new(path_output).join("list.html");
        self.write_content(&file_path, &replaced_content)?;

        Ok(())
    }

    fn generate_sites_table(
        &self,
        websites: &[Website],
    ) -> Result<String, Box<dyn std::error::Error>> {
        log::debug!("Generating webring list table...");

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
}
