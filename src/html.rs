use lazy_static::lazy_static;
use minify_html::{minify, Cfg};
use rand::prelude::SliceRandom;
use regex::Regex;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use crate::cli::AppSettings;
use opml::*;

//use crate::file::acquire_file_data;
use crate::website::WebringSite;

pub struct HtmlGenerator {
    tera: Tera,
    cfg: Cfg,
    skip_minify: bool,
}

// Stores pre-generated data for potential reuse
struct PrecomputedTags {
    table_of_sites: String,
    number_of_sites: usize,
    current_time: String,
    opml_link: String,
    featured_site_name: String,
    featured_site_description: String,
    featured_site_url: String,
}

impl HtmlGenerator {
    pub fn new(
        template_path: impl Into<PathBuf>,
        skip_minify: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cfg = Cfg::new();
        cfg.minify_css = true;
        cfg.minify_js = true;
        //cfg.keep_comments = true;

        let template_path_str = template_path
            .into()
            .join("**/*")
            .to_string_lossy()
            .to_string();
        let tera = Tera::new(&template_path_str)
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

        Ok(Self {
            tera,
            cfg,
            skip_minify,
        })
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

    pub async fn generate_opml(
        &self,
        webring: &[WebringSite],
        settings: &AppSettings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path_output = &settings.path_output;
        // Ensure output directory exists
        fs::create_dir_all(path_output)?;

        let mut opml = OPML::default();
        opml.head = Some(Head {
            title: Some(settings.ring_description.to_owned()),
            owner_name: Some(settings.ring_owner.to_owned()),
            owner_id: Some(settings.ring_owner_site.to_owned()),
            ..Head::default()
        });

        for (_index, website) in webring.iter().enumerate() {
            if let Some(owner) = &website.website.owner {
                if let Some(rss_url) = website.website.rss.as_ref().filter(|url| !url.is_empty()) {
                    opml.add_feed(owner, rss_url);
                }
            }
        }

        let mut file =
            std::fs::File::create(path_output.to_owned() + "/" + &settings.ring_name + ".opml")
                .unwrap();
        let _xml = opml.to_writer(&mut file).unwrap();

        Ok(())
    }

    pub async fn generate_html(
        &self,
        webring: &[WebringSite],
        settings: &AppSettings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path_output = &settings.path_output;
        // Ensure output directory exists
        fs::create_dir_all(path_output)?;

        let mut context = Context::new();
        context.insert("websites", webring);

        // Generate site-specific "next"/"previous" pages
        for site in webring.iter() {
            self.generate_site(site, webring, &context, path_output, settings)?;
        }

        // Process all other custom templates
        self.generate_custom_templates(&settings, &webring).await?;

        Ok(())
    }

    fn generate_site(
        &self,
        site: &WebringSite,
        webring: &[WebringSite],
        context: &Context,
        path_output: &str,
        settings: &AppSettings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create directory for the site
        let site_path = Path::new(path_output).join(&site.website.slug);
        
        fs::create_dir_all(&site_path.join(&settings.next_url_text))?;
        fs::create_dir_all(&site_path.join(&settings.prev_url_text))?;

        // Determine previous/next links
        let previous_site = &webring[site.previous].website.url;
        let next_site = &webring[site.next].website.url;

        let mut next_context = context.clone();
        next_context.insert("url", next_site);
        let content_next = self.tera.render("template.html", &next_context)?;
        self.write_content(&site_path.join(format!("{}/index.html", &settings.next_url_text)), &content_next)?;

        let mut previous_context = context.clone();
        previous_context.insert("url", previous_site);
        let content_previous = self.tera.render("template.html", &previous_context)?;
        self.write_content(&site_path.join(format!("{}/index.html", &settings.prev_url_text)), &content_previous)?;

        Ok(())
    }

    async fn generate_custom_templates(
        &self,
        settings: &AppSettings,
        webring: &[WebringSite],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Pre-generate expensive tag data for reuse
        let path_output = &settings.path_output;

        // Picking a random site for the featured site
        let mut rng = rand::thread_rng();
        let featured_site = webring.choose(&mut rng).unwrap();

        let precomputed = PrecomputedTags {
            table_of_sites: self.generate_sites_table(webring)?,
            number_of_sites: webring.len(),
            featured_site_url: featured_site.website.url.clone(),
            featured_site_name: featured_site.website.name.clone().unwrap_or_else(|| featured_site.website.url.clone()),
            featured_site_description: featured_site.website.about.clone().unwrap_or_else(|| "".to_string()),
            current_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            opml_link: "./".to_owned() + &settings.ring_name + ".opml",
        };

        // Load template files
        let template_paths = self
            .tera
            .get_template_names()
            .filter(|name| *name != "template.html");
        // TODO: Make this file path customizable 

        for template_name in template_paths {
            let context = self.process_tags(&precomputed, &settings)?;

            let template_file_name = Path::new(template_name)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();

            let content = self.tera.render(template_name, &context)?;
            let file_path = Path::new(path_output).join(template_file_name);
            self.write_content(&file_path, &content)?;
        }

        Ok(())
    }

    fn process_tags(
        &self,
        precomputed: &PrecomputedTags,
        settings: &AppSettings,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        let mut context = Context::new();

        // Process the "{{ table_of_sites }}" tag
        context.insert("table_of_sites", &precomputed.table_of_sites);
        // {{ base_url }}
        context.insert("base_url", &settings.base_url);
        // {{ ring_name }}
        context.insert("ring_name", &settings.ring_name);
        // {{ ring_description }}
        context.insert("ring_description", &settings.ring_description);
        // {{ ring_owner }}
        context.insert("ring_owner", &settings.ring_owner);
        // {{ ring_owner_site }}
        context.insert("ring_owner_site", &settings.ring_owner_site);
        // {{ number_of_sites }}
        context.insert("number_of_sites", &precomputed.number_of_sites);
        // {{ featured_site_name }}
        context.insert("featured_site_name", &precomputed.featured_site_name);
        // {{ featured_site_description }}
        context.insert("featured_site_description", &precomputed.featured_site_description);
        // {{ featured_site_url }}
        context.insert("featured_site_url", &precomputed.featured_site_url);
        // {{ current_time }}
        context.insert("current_time", &precomputed.current_time);
        // {{ opml }}
        context.insert("opml", &precomputed.opml_link);

        Ok(context)
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
            table_html.push_str(&website.website.slug);
            table_html.push_str("</td>\n");
            // URL
            table_html.push_str(&format!(
                "            <td><a href=\"{}\" target=\"_blank\">{}</a>",
                website.website.url, website.website.url
            ));
            if let Some(rss_url) = website.website.rss.as_ref().filter(|url| !url.is_empty()) {
                table_html.push_str(&format!(
                    " <a href=\"{}\" target=\"_blank\">[rss]</a>",
                    rss_url
                ));
            }
            table_html.push_str("</td>\n");
            // About
            table_html.push_str("            <td>");
            table_html.push_str(&website.website.about.as_deref().unwrap_or(""));
            table_html.push_str("</td>\n");
            // Owner
            if let Some(owner) = &website.website.owner {
                let formatted_owner = Self::format_owner(owner);
                table_html.push_str(&format!("            <td>{}</td>\n", formatted_owner));
            } else {
                // If owner is None, output an empty td
                table_html.push_str("            <td></td>\n");
            }
            table_html.push_str("        </tr>\n");
        }
        table_html.push_str("    </tbody>\n");

        // Close table tag
        table_html.push_str("</table>\n");

        Ok(table_html)
    }

    fn format_owner(owner: &str) -> String {
        owner
            .split_whitespace()
            .map(|part| {
                if HYPERLINK_REGEX.is_match(part) {
                    part.to_string()
                } else if let Some(caps) = FEDIVERSE_REGEX.captures(part) {
                    if caps.len() == 3 {
                        let username = &caps[1];
                        let domain = &caps[2];
                        format!("<a href=\"https://{}/@{}\">{}</a>", domain, username, part)
                    } else {
                        part.to_string()
                    }
                } else if PHONE_REGEX.is_match(part) {
                    format!("<a href=\"tel:{}\">{}</a>", part, part)
                } else if SMS_REGEX.is_match(part) {
                    format!("<a href=\"sms:{}\">{}</a>", part, part)
                } else if URL_REGEX.is_match(part) {
                    format!("<a href=\"{}\" target=\"_blank\">{}</a>", part, part)
                } else if EMAIL_REGEX.is_match(part) {
                    format!("<a href=\"mailto:{}\">{}</a>", part, part)
                } else {
                    part.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}

// pre-compile regex
lazy_static! {
    static ref HYPERLINK_REGEX: Regex =
        Regex::new(r#"<a\s+[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#).unwrap();
    static ref URL_REGEX: Regex = Regex::new(r"^[a-z]+://").unwrap();
    static ref EMAIL_REGEX: Regex =
        Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap();
    static ref FEDIVERSE_REGEX: Regex = Regex::new(r"^@([^\s@]+)@([^\s@]+\.[^\s@]+)$").unwrap();
    static ref PHONE_REGEX: Regex = Regex::new(r"^\+?\d{10,15}$").unwrap();
    static ref SMS_REGEX: Regex = Regex::new(r"^sms:\+?\d{10,15}$").unwrap();
}
