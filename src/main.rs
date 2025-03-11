use anyhow::Result;
use dotenv::dotenv;
use sitemaps::extract_sitemap_url_list;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};

mod client;
mod sitemaps;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let _psi_url = std::env::var("PSI_URL")?;
    let _psi_key = std::env::var("PSI_KEY")?;

    let websites = get_base_sites("sites.txt").await?;

    for url in websites.iter() {
        extract_sitemap_url_list(url).await?;
    }

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
