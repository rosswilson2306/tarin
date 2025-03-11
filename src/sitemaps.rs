use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use url::Url;

async fn fetch_sitemap(url: &str) -> Result<String> {
    // TODO: look into refactoring this into a client, see https://docs.rs/reqwest/latest/reqwest/?search=params#making-a-get-request
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch sitemaps: HTTP {}",
            response.status()
        ));
    }

    let body = response.text().await?;

    Ok(body)
}

fn build_sitemap_index_url(base_url: &str) -> Result<String> {
    let mut sitemap_url = Url::parse(base_url).context("Invalid base url")?;
    sitemap_url
        .path_segments_mut()
        .map_err(|_| anyhow!("Invalid base url: cannot set path segments"))?
        .push("sitemaps.xml");
    Ok(sitemap_url.to_string())
}

async fn extract_loc_urls(xml_string: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml_string);
    let mut urls: Vec<String> = Vec::new();

    while let Ok(event) = reader.read_event() {
        match event {
            Event::Start(ref e) if e.name().as_ref() == b"loc" => {
                if let Ok(text) = reader.read_text(e.name()) {
                    if Url::parse(&text).is_ok() {
                        urls.push(text.to_string());
                    }
                }
            }
            Event::Eof => break,
            _ => (),
        }
    }
    urls
}

fn remove_duplicates(mut source_set: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    source_set.retain(|x| seen.insert(x.clone()));
    source_set
}

pub async fn extract_sitemap_url_list(base_url: &str) -> Result<Vec<String>> {
    let sitemap_url =
        build_sitemap_index_url(base_url).context("Failed to build sitemap index url")?;
    let sitemap = fetch_sitemap(&sitemap_url)
        .await
        .context("Unable to fetch sitemap")?;

    let mut all_urls = Vec::new();

    let top_level_urls = extract_loc_urls(&sitemap).await;
    let unique_top_level_urls = remove_duplicates(top_level_urls);

    for top_level_url in unique_top_level_urls {
        // TODO: this should come from a config file
        if top_level_url.contains("locations") || top_level_url.contains("counties") {
            continue;
        }

        if top_level_url.contains("sitemaps") {
            let child_sitemap_url = fetch_sitemap(&top_level_url).await?;
            let child_urls = extract_loc_urls(&child_sitemap_url).await;
            all_urls.extend(child_urls);
        }
    }

    Ok(all_urls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use std::error::Error;
    use tokio::test;

    #[test]
    async fn add_sitemap_index_path() -> Result<(), Box<dyn Error>> {
        let base_url = "https://example.com";
        let sitemap_url = build_sitemap_index_url(base_url)?;

        assert_eq!(sitemap_url, "https://example.com/sitemaps.xml");

        Ok(())
    }

    #[test]
    async fn build_request_for_sitemap() -> Result<(), Box<dyn Error>> {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/sitemaps.xml")
            .with_body("<xml>Mock Sitemap</xml>")
            .create_async()
            .await;

        let url = build_sitemap_index_url(&server.url())?;
        let response = fetch_sitemap(&url).await?;

        mock.assert_async().await;
        assert_eq!(response, "<xml>Mock Sitemap</xml>");

        Ok(())
    }

    #[test]
    async fn fail_when_url_invalid() {
        let result = fetch_sitemap("invalid-url").await;
        assert!(result.is_err());
    }

    #[test]
    async fn fail_on_invalid_protocal() {
        let result = fetch_sitemap("ftp://invalid").await;
        assert!(result.is_err());
    }

    #[test]
    async fn not_found_sitemaps() -> Result<(), Box<dyn Error>> {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/sitemaps.xml")
            .with_status(404)
            .with_body("Not Found")
            .create_async()
            .await;

        let url = build_sitemap_index_url(&server.url())?;
        let result = fetch_sitemap(&url).await;

        assert!(result.is_err());
        mock.assert_async().await;
        Ok(())
    }

    #[test]
    async fn remove_duplicate_urls() {
        let urls = vec![
            "https://example.com".to_string(),
            "https://example-2.com".to_string(),
            "https://example.com".to_string(),
            "https://example-3.com".to_string(),
            "https://example-2.com".to_string(),
        ];

        let unique = remove_duplicates(urls);

        assert_eq!(
            unique,
            vec![
                "https://example.com".to_string(),
                "https://example-2.com".to_string(),
                "https://example-3.com".to_string()
            ]
        );
    }

    #[test]
    async fn extracts_valid_loc_urls() {
        let xml = r#"
        <urlset>
            <url>
                <loc>https://example.com/page1</loc>
            </url>
            <url>
                <loc>https://example.com/page2</loc>
            </url>
        </urlset>
        "#;

        let urls = extract_loc_urls(xml).await;
        assert_eq!(
            urls,
            vec!["https://example.com/page1", "https://example.com/page2"]
        );
    }

    #[test]
    async fn skips_invalid_urls() {
        let xml = r#"
        <urlset>
            <url>
                <loc>https://example.com/page1</loc>
            </url>
            <url>
                <loc>invalid_url</loc>
            </url>
        </urlset>
        "#;

        let urls = extract_loc_urls(xml).await;
        assert_eq!(urls, vec!["https://example.com/page1"]);
    }
}
