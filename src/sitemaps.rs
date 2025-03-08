use anyhow::{anyhow, Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use url::Url;

pub async fn fetch_sitemap(url: &str) -> Result<String> {
    let mut sitemap_url = Url::parse(url).context("Invalid base url")?;
    sitemap_url
        .path_segments_mut()
        .map_err(|_| anyhow!("Invalid base url: cannot set path segments"))?
        .push("sitemaps.xml");

    // TODO: look into refactoring this into a client, see https://docs.rs/reqwest/latest/reqwest/?search=params#making-a-get-request
    let response = reqwest::get(sitemap_url).await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch sitemaps: HTTP {}",
            response.status()
        ));
    }

    let body = response.text().await?;

    Ok(body)
}

pub async fn extract_loc_urls(xml_string: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use std::error::Error;
    use tokio::test;

    #[test]
    async fn build_request_for_sitemap() -> Result<(), Box<dyn Error>> {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/sitemaps.xml")
            .with_body("<xml>Mock Sitemap</xml>")
            .create_async()
            .await;

        let response = fetch_sitemap(&server.url()).await?;

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
    async fn not_found_sitemaps() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/sitemaps.xml")
            .with_status(404)
            .with_body("Not Found")
            .create_async()
            .await;

        let result = fetch_sitemap(&server.url()).await;
        println!("{:?}", result);

        assert!(result.is_err());
        mock.assert_async().await;
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
