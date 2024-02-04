use minify_html::{minify, Cfg};
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::file::acquire_file_data;
use crate::website::{Website, WebringSite};

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
        websites: &[WebringSite],
        path_output: &str,
        path_template_redirects: &str,
        path_template_index: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get template file for redirect pages
        log::debug!("Attempting to load webring redirect HTML template...");
        let template_redirect = self.acquire_template(path_template_redirects).await?;

        for webring_site in websites.iter() {
            let previous_site = &websites[webring_site.previous].website;
            let next_site = &websites[webring_site.next].website;

            // Generate HTML for this website
            self.generate_html(webring_site, previous_site, next_site, path_output, &template_redirect)?;
        }

        self.generate_list_html(websites, path_output, path_template_index)
            .await?;

        Ok(())
    }

    fn generate_html(
        &self,
        website: &WebringSite,
        previous_site: &Website,
        next_site: &Website,
        path_output: &str,
        template_redirect: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let directory_path = format!("{}/{}", path_output, website.website.name);
        fs::create_dir_all(&directory_path)?;

        let next_html_path = Path::new(&directory_path).join("next.html");
        let previous_html_path = Path::new(&directory_path).join("previous.html");
        
        let next_html_content = template_redirect.replace(
            "<!-- REDIRECT -->",
            &format!(
                "<meta http-equiv=\"refresh\" content=\"0; url={}\">",
                next_site.url
            ),
        );
        let previous_html_content = template_redirect.replace(
            "<!-- REDIRECT -->",
            &format!(
                "<meta http-equiv=\"refresh\" content=\"0; url={}\">",
                previous_site.url
            ),
        );

        self.write_content(&previous_html_path, &previous_html_content)?;
        self.write_content(&next_html_path, &next_html_content)?;

        Ok(())
    }

    async fn generate_list_html(
        &self,
        websites: &[WebringSite],
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
        websites: &[WebringSite],
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
            // Index #
            table_html.push_str(&format!("            <td>{}</td>\n", index + 1));
            // Name
            table_html.push_str("            <td>");
            table_html.push_str(&website.website.name);
            table_html.push_str("</td>\n");
            // URL
            table_html.push_str(&format!("            <td><a href=\"{}\" target=\"_blank\">{}</a>", website.website.url, website.website.url));
        if let Some(rss_url) = website.website.rss.as_ref().filter(|url| !url.is_empty()) {
            table_html.push_str(&format!(" <a href=\"{}\" target=\"_blank\">[rss]</a>", rss_url));
        }
        table_html.push_str("</td>\n");
            // About
            table_html.push_str("            <td>");
            table_html.push_str(&website.website.about);
            table_html.push_str("</td>\n");
            // Owner
            table_html.push_str("            <td>");
            table_html.push_str(&website.website.owner);
            table_html.push_str("</td>\n");
            table_html.push_str("        </tr>\n");
        }
        table_html.push_str("    </tbody>\n");

        // Close table tag
        table_html.push_str("</table>\n");

        Ok(table_html)
    }
}
