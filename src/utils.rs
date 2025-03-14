use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};

// Dev helper to get website list from file
pub async fn get_base_sites(filepath: &str) -> io::Result<Vec<String>> {
    let file = File::open(filepath).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut sites = Vec::new();

    while let Some(line) = lines.next_line().await? {
        sites.push(line)
    }

    Ok(sites)
}
