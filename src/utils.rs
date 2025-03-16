use anyhow::Result;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use url::Url;

// Dev helper to get website list from file
pub async fn get_base_sites(filepath: &str) -> Result<Vec<Url>> {
    let file = File::open(filepath).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut sites = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let site_url = Url::parse(&line)?;
        sites.push(site_url)
    }

    Ok(sites)
}
