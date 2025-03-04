use anyhow::{anyhow, Result};
use url::Url;

pub async fn fetch_sitemap(url: &str) -> Result<String> {
    let mut sitemap_url = Url::parse(url)?;
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
}
