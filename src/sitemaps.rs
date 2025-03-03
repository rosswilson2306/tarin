use std::error::Error;

pub async fn fetch_site_map(url: &str) -> Result<String, Box<dyn Error>> {
    let sitemap_url = format!("{}/sitemaps.xml", url);
    let response = reqwest::get(sitemap_url).await?.text().await?;
    Ok(response)
}
