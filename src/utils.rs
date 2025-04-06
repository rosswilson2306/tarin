use anyhow::Result;
use url::Url;

use crate::config::load_config;

// Dev helper to get website list from file
pub async fn get_base_sites() -> Result<Vec<Url>> {
    let mut sites = Vec::new();

    if let Some(config) = load_config("config.toml").await {
        for site in &config.sites {
            let site_url = Url::parse(site)?;
            sites.push(site_url)
        }
    } else {
        panic!();
    }

    Ok(sites)
}
