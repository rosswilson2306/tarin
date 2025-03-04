use client::PsiClient;
use dotenv::dotenv;
use sitemaps::fetch_sitemap;
use std::error::Error;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};

mod client;
mod sitemaps;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let psi_url = std::env::var("PSI_URL")?;
    let psi_key = std::env::var("PSI_KEY")?;

    let websites = get_base_sites("sites.txt").await?;

    for url in websites.iter() {
        let _sitemap = fetch_sitemap(url).await?;
    }

    let client = PsiClient::new(&psi_url, &psi_key);

    let _report = client.get_report("https://google.com").await?;

    Ok(())
}

// Dev helper to get website list from file
async fn get_base_sites(filepath: &str) -> io::Result<Vec<String>> {
    let file = File::open(filepath).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut sites = Vec::new();

    while let Some(line) = lines.next_line().await? {
        sites.push(line)
    }

    Ok(sites)
}
