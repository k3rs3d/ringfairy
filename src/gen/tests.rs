use super::*;
use crate::cli::AppSettings;
use crate::gen::html::*;
use crate::gen::webring::*;
use crate::http;
use crate::website;
use crate::website::*;
use std::path::PathBuf;

// Webring

fn create_sample_website(slug: &str, url: &str) -> Website {
    Website {
        slug: slug.to_string(),
        name: Some(format!("Site {}", slug)),
        about: Some(format!("About {}", slug)),
        url: url.to_string(),
        rss: Some(format!("http://{}.tld/rss", slug)),
        owner: Some(format!("Owner {}", slug)),
        misc: None,
    }
}

fn build_settings() -> AppSettings {
    AppSettings {
        shuffle: false,
        ..Default::default()
    }
}

fn build_settings_no_shuffle() -> AppSettings {
    AppSettings {
        shuffle: true,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_build_webring() {
    // sample data
    let websites = vec![
        create_sample_website("site1", "https://site1.tld"),
        create_sample_website("site2", "https://site2.tld"),
        create_sample_website("site3", "https://site3.tld"),
    ];

    let webring_sites = build_webring_sequence(websites.clone(), &build_settings()).await;

    assert_eq!(webring_sites.len(), 3);

    assert_eq!(webring_sites[0].website.slug, "site1");
    assert_eq!(webring_sites[0].next, 1);
    assert_eq!(webring_sites[0].previous, 2);

    assert_eq!(webring_sites[1].website.slug, "site2");
    assert_eq!(webring_sites[1].next, 2);
    assert_eq!(webring_sites[1].previous, 0);

    assert_eq!(webring_sites[2].website.slug, "site3");
    assert_eq!(webring_sites[2].next, 0);
    assert_eq!(webring_sites[2].previous, 1);
}

#[tokio::test]
async fn test_build_webring_shuffle() {
    // sample data
    let websites = vec![
        create_sample_website("site1", "https://site1.tld"),
        create_sample_website("site2", "https://site2.tld"),
        create_sample_website("site3", "https://site3.tld"),
    ];

    // shuffle enabled
    let webring_sites =
        build_webring_sequence(websites.clone(), &build_settings_no_shuffle()).await;

    assert_eq!(webring_sites.len(), 3);

    let mut slugs: Vec<&str> = webring_sites
        .iter()
        .map(|site| site.website.slug.as_str())
        .collect();
    slugs.sort();
    let expected_slugs: Vec<&str> = websites.iter().map(|site| site.slug.as_str()).collect();
    assert_eq!(slugs, expected_slugs);

    for i in 0..webring_sites.len() {
        let next_index = (i + 1) % webring_sites.len();
        assert_eq!(webring_sites[i].next, next_index);

        let prev_index = if i == 0 {
            webring_sites.len() - 1
        } else {
            i - 1
        };
        assert_eq!(webring_sites[i].previous, prev_index);
    }
}

#[tokio::test]
async fn test_verify_websites_valid() {
    let websites = vec![
        create_sample_website("site1", "https://site1.tld"),
        create_sample_website("site2", "https://site2.tld"),
        create_sample_website("site3", "https://site3.tld"),
    ];

    let result = verify_websites(&websites);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_verify_duplicate_slugs() {
    let websites = vec![
        create_sample_website("site1", "https://site1.tld"),
        create_sample_website("site1", "https://site2.tld"),
    ];

    let result = verify_websites(&websites);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_duplicate_urls() {
    let websites = vec![
        create_sample_website("site1", "https://site1.tld"),
        create_sample_website("site2", "https://site1.tld"),
    ];

    let result = verify_websites(&websites);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_empty_url() {
    let websites = vec![create_sample_website("site1", "")];

    let result = verify_websites(&websites);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_invalid_url() {
    let websites = vec![create_sample_website("site1", "htp/invalid-url")];

    let result = verify_websites(&websites);
    assert!(result.is_err());
}

// HTML

// Mock data
fn mock_webring_site() -> Vec<WebringSite> {
    vec![
        WebringSite {
            website: Website {
                slug: "site1".to_string(),
                name: Some("Site 1".to_string()),
                url: "https://site1.com".to_string(),
                about: Some("About Site 1".to_string()),
                owner: Some("owner1".to_string()),
                rss: Some("https://site1.com/rss".to_string()),
                misc: None,
            },
            previous: 1,
            next: 1,
        },
        WebringSite {
            website: Website {
                slug: "site2".to_string(),
                name: Some("Site 2".to_string()),
                url: "https://site2.com".to_string(),
                about: Some("About Site 2".to_string()),
                owner: Some("owner2".to_string()),
                rss: Some("https://site2.com/rss".to_string()),
                misc: None,
            },
            previous: 0,
            next: 0,
        },
    ]
}

fn mock_app_settings() -> AppSettings {
    AppSettings {
        path_output: "output".to_string(),
        base_url: "https://example.com".to_string(),
        ring_name: "Test Ring".to_string(),
        ring_description: "Description for Test Ring".to_string(),
        ring_owner: "Test Owner".to_string(),
        ring_owner_site: "https://owner.com".to_string(),
        next_url_text: "next".to_string(),
        prev_url_text: "prev".to_string(),
        ..Default::default()
    }
}

#[tokio::test]
async fn test_html_generator_new() {
    let template_path = PathBuf::from("templates");
    let generator = HtmlGenerator::new(template_path, false).await;

    assert!(generator.is_ok());
}

#[tokio::test]
async fn test_precompute_tags() {
    let webring = mock_webring_site();
    let settings = mock_app_settings();

    let tags = HtmlGenerator::precompute_tags(&webring, &settings).await;

    assert_eq!(tags.number_of_sites, 2);
    assert!(tags.featured_site_name == "Site 1" || tags.featured_site_name == "Site 2");
}

#[tokio::test]
async fn test_ensure_output_directory() {
    let path = "output_dir";

    let generator = HtmlGenerator::new(PathBuf::from("templates"), false)
        .await
        .unwrap();
    let result = generator.ensure_output_directory(path).await;

    assert!(result.is_ok());
    assert!(Path::new(path).exists());

    fs::remove_dir_all(path).unwrap();
}

#[tokio::test]
async fn test_audit_websites() {
    // Mock settings and website
    let settings = mock_app_settings();
    let mut mock_site = create_sample_website("test", "");

    // Mock HTTP server and client
    let mut mock_server = mockito::Server::new_async().await; // mockito server
    mock_site.url = mock_server.url();
    let mock = mock_server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(
            r#"<a href="https://example.com/test/prev/">←</a>
    <a href="https://example.com/">Test Ring</a>
    <a href="https://example.com/test/next/">→</a>"#,
        )
        .create();
    let audit_client = http::setup_client(&settings).await.unwrap(); // reqwest client

    let audit_result = website::does_html_contain_links(&audit_client, &mock_site, &settings).await;

    mock.assert_async().await; // Verify that mock was called
    assert!(audit_result.is_ok()); // Mock response should return Ok

    // TODO call audit function website::audit_links
    //        -> verify function returns correctly audited sites
}
