use minify_html::{minify, Cfg};
use opml::{Head, OPML};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

use super::*;
use crate::cli::AppSettings;
use crate::error::Error;
use crate::gen::{webring::WebringSite, webring::WebringSiteList, Generator, PrecomputedTags};

pub struct HtmlGenerator {
    tera: Tera,
    cfg: Cfg,
    skip_minify: bool,
}

impl Generator for HtmlGenerator {
    async fn new(template_path: PathBuf, skip_minify: bool) -> Result<Self, Error> {
        let mut cfg = Cfg::new();
        cfg.minify_css = true;
        cfg.minify_js = true;

        let template_path_str = template_path.join("**/*").to_string_lossy().to_string();
        let tera = Tera::new(&template_path_str)?;
        Ok(Self {
            tera,
            cfg,
            skip_minify,
        })
    }

    async fn write_content(&self, file_path: &Path, content: &str) -> Result<(), Error> {
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

    async fn generate_content(
        &self,
        webring: &WebringSiteList,
        settings: &AppSettings,
    ) -> Result<(), Error> {
        self.ensure_output_directory(&settings.path_output).await?;
        let precomputed = <HtmlGenerator as Generator>::precompute_tags(webring, settings).await;
        let context = self
            .generate_context(&webring, &precomputed, settings)
            .await?;

        self.generate_html(&webring, settings, &context).await?;
        self.generate_opml(&webring.sites, settings).await?;

        Ok(())
    }
}

impl HtmlGenerator {
    async fn generate_html(
        &self,
        webring: &WebringSiteList,
        settings: &AppSettings,
        context: &Context,
    ) -> Result<(), Error> {
        // Generate site-specific pages
        for site in &webring.sites {
            self.generate_site(&site, webring, context, &settings.path_output, settings)
                .await?;
        }

        // Process all other custom templates
        self.generate_custom_templates(settings, webring).await?;
        Ok(())
    }

    async fn generate_opml(
        &self,
        webring: &[WebringSite],
        settings: &AppSettings,
    ) -> Result<(), Error> {
        log::info!("Generating OPML file...");

        let path_output = &settings.path_output;
        fs::create_dir_all(path_output)?;

        let mut opml = OPML {
            head: Some(Head {
                title: Some(settings.ring_description.to_owned()),
                owner_name: Some(settings.ring_owner.to_owned()),
                owner_id: Some(settings.ring_owner_site.to_owned()),
                ..Head::default()
            }),
            ..OPML::default()
        };

        for website in webring.iter() {
            if let Some(owner) = &website.website.owner {
                if let Some(rss_url) = website.website.rss.as_ref().filter(|url| !url.is_empty()) {
                    opml.add_feed(owner, rss_url);
                }
            }
        }

        let mut file =
            std::fs::File::create(path_output.to_owned() + "/" + &settings.ring_name + ".opml")
                .unwrap();

        opml.to_writer(&mut file).unwrap();

        log::info!("OPML file generated.");
        Ok(())
    }

    async fn generate_site(
        &self,
        site: &WebringSite,
        webring: &WebringSiteList,
        context: &Context,
        path_output: &str,
        settings: &AppSettings,
    ) -> Result<(), Error> {
        let site_path = Path::new(path_output).join(&site.website.slug);
        fs::create_dir_all(site_path.join(&settings.next_url_text))?;
        fs::create_dir_all(site_path.join(&settings.prev_url_text))?;

        let previous_site = &webring.sites[site.previous].website.url;
        let next_site = &webring.sites[site.next].website.url;

        self.render_and_write(
            &site_path,
            &settings.next_url_text,
            next_site,
            &settings.filename_template_redirect,
            context,
        )
        .await?;
        self.render_and_write(
            &site_path,
            &settings.prev_url_text,
            previous_site,
            &settings.filename_template_redirect,
            context,
        )
        .await?;

        Ok(())
    }

    async fn render_and_write(
        &self,
        site_path: &Path,
        url_text: &str,
        site_url: &str,
        template_name: &str,
        context: &Context,
    ) -> Result<(), Error> {
        let mut url_context = context.clone();
        url_context.insert("url", site_url);

        let content = self.tera.render(template_name, &url_context)?;
        self.write_content(
            &site_path.join(format!("{}/index.html", url_text)),
            &content,
        )
        .await?;

        Ok(())
    }

    async fn generate_custom_templates(
        &self,
        settings: &AppSettings,
        webring: &WebringSiteList,
    ) -> Result<(), Error> {
        let path_output = &settings.path_output;

        let precomputed = <HtmlGenerator as Generator>::precompute_tags(webring, settings).await;

        for template_name in self.tera.get_template_names().filter(|name| {
            *name != settings.filename_template_redirect
        }) {
            let context = self
                .generate_context(webring, &precomputed, settings)
                .await?;
            let content = self.tera.render(template_name, &context)?;
            let file_path = Path::new(path_output).join(template_name);
            self.write_content(&file_path, &content).await?;
        }
        Ok(())
    }

    async fn generate_context(
        &self,
        webring: &WebringSiteList,
        precomputed: &PrecomputedTags,
        settings: &AppSettings,
    ) -> Result<Context, Error> {
        let mut context = Context::new();
        // Many of these are redundant
        // Keeping them for compatibility (for now)
        context.insert("table_of_sites", &build_sites_table_html(&webring.sites).await);
        context.insert("grid_of_sites", &build_sites_grid_html(&webring.sites).await);
        context.insert("base_url", &settings.base_url);
        context.insert("ring_name", &settings.ring_name);
        context.insert("ring_description", &settings.ring_description);
        context.insert("ring_owner", &settings.ring_owner);
        context.insert("ring_owner_site", &settings.ring_owner_site);
        context.insert("number_of_sites", &precomputed.number_of_sites);
        context.insert("featured_site_name", &precomputed.featured_site_name);
        context.insert(
            "featured_site_description",
            &precomputed.featured_site_description,
        );
        context.insert("featured_site_url", &precomputed.featured_site_url);
        context.insert("current_time", &precomputed.current_time);
        context.insert("opml", &precomputed.opml_link);
        context.insert("sites", &webring.sites); // Insert the whole list
        context.insert("failed_sites", &webring.failed_sites);

        Ok(context)
    }
}

pub async fn build_sites_table_html(websites: &[WebringSite]) -> String {
    // HTML-specific table generation
    let mut table_html = String::new();
    table_html.push_str("<table>\n<thead>\n<tr>\n");
    table_html.push_str("<th scope=\"col\">#</th>\n<th scope=\"col\">Name</th>\n<th scope=\"col\">URL</th>\n<th scope=\"col\">About</th>\n<th scope=\"col\">Owner</th>\n");
    table_html.push_str("</tr>\n</thead>\n<tbody>\n");
    for (index, website) in websites.iter().enumerate() {
        table_html.push_str("<tr>\n");
        table_html.push_str(&format!("<td>{}</td>\n", index + 1));
        table_html.push_str(&format!("<td>{}</td>\n", website.website.slug));
        table_html.push_str(&format!(
            "<td><a href=\"{}\" target=\"_blank\">{}</a>{}</td>\n",
            website.website.url,
            website.website.url,
            if let Some(rss_url) = &website.website.rss {
                format!(" <a href=\"{}\" target=\"_blank\">[rss]</a>", rss_url)
            } else {
                String::new()
            }
        ));
        table_html.push_str(&format!(
            "<td>{}</td>\n",
            website.website.about.as_deref().unwrap_or("")
        ));
        table_html.push_str(&format!(
            "<td>{}</td>\n",
            website
                .website
                .owner
                .as_deref()
                .map(format_owner)
                .unwrap_or(String::new())
        ));
        table_html.push_str("</tr>\n");
    }
    table_html.push_str("</tbody>\n</table>\n");

    table_html
}

pub async fn build_sites_grid_html(websites: &[WebringSite]) -> String {
    // Layout using CSS grid
    let mut grid_html = String::new();
    grid_html.push_str("<section class=\"cards\">\n");
    for (_index, website) in websites.iter().enumerate() {
        grid_html.push_str("<article class=\"card\">\n");
        grid_html.push_str(&format!(
            "<div class=\"card-name\">{} <span class=\"card-slug\">({})</span></div>\n",
            website
                .website
                .owner
                .as_deref()
                .map(format_owner)
                .unwrap_or(String::new()),
            website.website.slug
        ));
        grid_html.push_str("<div class=\"card-content\">\n");
        grid_html.push_str(&format!(
            "<div class=\"card-link\"><a href=\"{}\" target=\"_blank\">{}</a>&nbsp;{}</div>\n",
            website.website.url,
            website.website.url,
            if let Some(rss_url) = &website.website.rss {
                format!(" <a href=\"{}\" target=\"_blank\">[rss]</a>", rss_url)
            } else {
                String::new()
            }
        ));
        grid_html.push_str(&format!(
            "<div class=\"card-text\">{}</div>\n",
            website.website.about.as_deref().unwrap_or("")
        ));
        grid_html.push_str("</div>\n"); //div card-content
        grid_html.push_str("</article>\n");
    }
    grid_html.push_str("</section>");
    grid_html
}
// TODO: make async?
pub fn format_owner(owner: &str) -> String {
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
