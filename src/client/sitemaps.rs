use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use url::Url;

use crate::config::{load_config, Config};

async fn fetch_sitemap(url: &Url) -> Result<String> {
    // TODO: look into refactoring this into a client, see https://docs.rs/reqwest/latest/reqwest/?search=params#making-a-get-request
    let response = reqwest::get(url.to_string()).await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch sitemaps: HTTP {}",
            response.status()
        ));
    }

    let body = response.text().await?;

    Ok(body)
}

fn build_sitemap_index_url(base_url: &Url) -> Result<Url> {
    let mut url = base_url.clone();
    url.path_segments_mut()
        .map_err(|_| anyhow!("Invalid base url: cannot set path segments"))?
        .push("sitemaps.xml");
    Ok(url)
}

async fn extract_loc_urls(xml_string: &str) -> Vec<Url> {
    let mut reader = Reader::from_str(xml_string);
    let mut urls: Vec<Url> = Vec::new();

    while let Ok(event) = reader.read_event() {
        match event {
            Event::Start(ref e) if e.name().as_ref() == b"loc" => {
                if let Ok(text) = reader.read_text(e.name()) {
                    let url = Url::parse(&text);

                    if let Ok(url) = url {
                        urls.push(url);
                    }
                }
            }
            Event::Eof => break,
            _ => (),
        }
    }
    urls
}

fn remove_duplicates(mut source_set: Vec<Url>) -> Vec<Url> {
    let mut seen = HashSet::new();
    source_set.retain(|x| seen.insert(x.clone()));
    source_set
}

pub async fn extract_sitemap_url_list(base_url: &Url) -> Result<Vec<Url>> {
    let config = load_config("config.toml").await;

    let sitemap_index_url =
        build_sitemap_index_url(base_url).context("Failed to build sitemap index url")?;

    let mut visited_sitemaps = HashSet::new();
    let mut all_urls = Vec::new();

    fetch_sitemap_recurse(
        &sitemap_index_url,
        &mut all_urls,
        &mut visited_sitemaps,
        &config,
    )
    .await?;

    Ok(all_urls)
}

#[async_recursion]
async fn fetch_sitemap_recurse(
    sitemap_url: &Url,
    all_urls: &mut Vec<Url>,
    visited_sitemaps: &mut HashSet<Url>,
    config: &Option<Config>,
) -> Result<()> {
    if !visited_sitemaps.insert(sitemap_url.to_owned()) {
        return Ok(());
    }

    let sitemap = fetch_sitemap(sitemap_url)
        .await
        .context("Unable to fetch sitemap")?;

    let urls = extract_loc_urls(&sitemap).await;
    let unique_urls = remove_duplicates(urls);
    let mut seen_patterns = HashSet::new();

    for url in unique_urls {
        if let Some(ref config) = config {
            if config
                .ignore_paths
                .iter()
                .any(|ignore_path| url.path().contains(ignore_path))
            {
                println!("Matched ignore list: {url}");
                continue;
            }

            if let Some(matched_pattern) = get_pattern(&url, &config.patterns) {
                if seen_patterns.contains(&matched_pattern) {
                    println!("Skipping duplicate pattern: {url}");
                    continue;
                }
                seen_patterns.insert(matched_pattern.clone());
            }
        }

        if url.path().contains("sitemaps/") {
            fetch_sitemap_recurse(&url, all_urls, visited_sitemaps, config).await?;
        } else {
            all_urls.push(url);
        }
    }

    Ok(())
}

fn get_pattern(url: &Url, patterns: &Vec<String>) -> Option<String> {
    for pattern in patterns {
        if let Some(matched) = match_dynamic_url_pattern(url, pattern) {
            return Some(matched);
        }
    }
    None
}

fn match_dynamic_url_pattern(url: &Url, pattern: &str) -> Option<String> {
    let pattern_parts: Vec<&str> = pattern.split("/").filter(|s| !s.is_empty()).collect();
    let url_parts: Vec<&str> = url.path_segments()?.collect();

    if url_parts.len() != pattern_parts.len() {
        return None;
    }

    for (p_part, u_part) in pattern_parts.iter().zip(url_parts.iter()) {
        if p_part.starts_with(":") {
            continue;
        }

        if p_part != u_part {
            return None;
        }
    }

    Some(pattern.to_string())
}

#[cfg(test)]
mod tests {
    mod build_sitemap_index_url {
        use super::super::*;
        use std::error::Error;

        #[tokio::test]
        async fn add_sitemap_index_path() -> Result<(), Box<dyn Error>> {
            let base_url = Url::parse("https://example.com")?;
            let sitemap_url = build_sitemap_index_url(&base_url)?;

            let result = Url::parse("https://example.com/sitemaps.xml")?;
            assert_eq!(sitemap_url, result);

            Ok(())
        }
    }

    mod fetch_sitemap {
        use super::super::*;
        use mockito::Server;
        use std::error::Error;

        #[tokio::test]
        async fn build_request_for_sitemap() -> Result<(), Box<dyn Error>> {
            let mut server = Server::new_async().await;
            let mock = server
                .mock("GET", "/sitemaps.xml")
                .with_body("<xml>Mock Sitemap</xml>")
                .create_async()
                .await;

            let server_url = Url::parse(&server.url())?;
            let url = build_sitemap_index_url(&server_url)?;
            let response = fetch_sitemap(&url).await?;

            mock.assert_async().await;
            assert_eq!(response, "<xml>Mock Sitemap</xml>");

            Ok(())
        }

        #[tokio::test]
        async fn not_found_sitemaps() -> Result<(), Box<dyn Error>> {
            let mut server = Server::new_async().await;
            let mock = server
                .mock("GET", "/sitemaps.xml")
                .with_status(404)
                .with_body("Not Found")
                .create_async()
                .await;

            let server_url = Url::parse(&server.url())?;
            let url = build_sitemap_index_url(&server_url)?;
            let result = fetch_sitemap(&url).await;

            assert!(result.is_err());
            mock.assert_async().await;
            Ok(())
        }
    }

    mod remove_duplicates {
        use super::super::*;
        use std::error::Error;

        #[tokio::test]
        async fn remove_duplicate_urls() -> Result<(), Box<dyn Error>> {
            let url1 = Url::parse("https://example.com")?;
            let url2 = Url::parse("https://example-2.com")?;
            let url3 = Url::parse("https://example-3.com")?;
            let urls = vec![
                url1.clone(),
                url2.clone(),
                url1.clone(),
                url3.clone(),
                url2.clone(),
            ];

            let unique = remove_duplicates(urls);

            assert_eq!(unique, vec![url1, url2, url3,]);

            Ok(())
        }
    }

    mod extract_loc_urls {
        use super::super::*;
        use std::error::Error;

        #[tokio::test]
        async fn extracts_valid_loc_urls() -> Result<(), Box<dyn Error>> {
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
            let url1 = Url::parse("https://example.com/page1")?;
            let url2 = Url::parse("https://example.com/page2")?;
            assert_eq!(urls, vec![url1, url2]);

            Ok(())
        }

        #[tokio::test]
        async fn skips_invalid_urls() -> Result<(), Box<dyn Error>> {
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
            let url1 = Url::parse("https://example.com/page1")?;
            assert_eq!(urls, vec![url1]);
            Ok(())
        }
    }

    mod match_dynamic_url_pattern {
        use super::super::*;
        use std::error::Error;

        #[tokio::test]
        async fn url_segment_length_less_than_pattern_length() -> Result<(), Box<dyn Error>> {
            let url = Url::parse("https://example.com/foo/123")?;
            let pattern = "/foo/bar/:id";

            let output = match_dynamic_url_pattern(&url, pattern);
            assert!(output.is_none());
            Ok(())
        }

        #[tokio::test]
        async fn fail_if_static_url_segment_differs_from_pattern() -> Result<(), Box<dyn Error>> {
            let url = Url::parse("https://examples.com/foo/123")?;
            let pattern = "/bar/:id";

            let output = match_dynamic_url_pattern(&url, pattern);
            assert!(output.is_none());
            Ok(())
        }

        #[tokio::test]
        async fn match_url_to_pattern() -> Result<(), Box<dyn Error>> {
            let url = Url::parse("https://examples.com/example/123")?;
            let pattern = "/example/:id";

            let output = match_dynamic_url_pattern(&url, pattern);
            assert!(output.is_some());
            Ok(())
        }
    }

    mod get_pattern {
        use super::super::*;
        use std::error::Error;

        #[test]
        fn no_pattern_found() -> Result<(), Box<dyn Error>> {
            let url = Url::parse("https://example.com/foo/bar/baz")?;
            let patterns = vec!["/bar/bat".to_string(), "/bar/:id".to_string()];

            let result = get_pattern(&url, &patterns);
            assert!(result.is_none());

            Ok(())
        }

        #[test]
        fn matched_pattern_found() -> Result<(), Box<dyn Error>> {
            let url = Url::parse("https://example.com/foo/123")?;
            let patterns = vec!["/foo/:id".to_string(), "/bar/:slug".to_string()];

            let pattern = get_pattern(&url, &patterns).ok_or("Not found")?;
            assert_eq!(pattern, "/foo/:id");
            Ok(())
        }
    }
}
