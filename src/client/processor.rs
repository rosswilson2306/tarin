use anyhow::{anyhow, Result};
use axum::response::sse::Event;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::{sync::mpsc, task, time::sleep};

use crate::{
    client::{psi::PsiClient, sitemaps::extract_sitemap_url_list},
    utils::get_base_sites,
};

pub async fn process_websites(
    sender: mpsc::Sender<std::result::Result<Event, Infallible>>,
) -> Result<()> {
    let websites = get_base_sites().await?;

    let psi_key = match std::env::var("PSI_KEY") {
        Ok(key) => key,
        Err(e) => return Err(anyhow!("Missing Page Speed Insights API key: {}", e)),
    };

    let psi_url = match std::env::var("PSI_URL") {
        Ok(url) => url,
        Err(e) => return Err(anyhow!("Missing Page Speed Insights URL: {}", e)),
    };

    let psi_client = Arc::new(PsiClient::new(&psi_url, &psi_key));

    for site in websites {
        let sender = sender.clone();
        let psi_client = psi_client.clone();

        task::spawn(async move {
            let site_urls = match extract_sitemap_url_list(&site).await {
                Ok(urls_list) => urls_list,
                Err(_) => {
                    return eprintln!(
                        "Failed to extract urls from sitemaps for: {}",
                        site.as_ref()
                    )
                }
            };

            for url in site_urls {
                println!("Running PSI report for: {} ...", url);

                let psi_res = match psi_client.get_report(url.as_ref()).await {
                    Ok(res) => res,
                    Err(_) => {
                        eprintln!("Error fetching report for: {}", url);
                        break;
                    }
                };

                let psi_event = match Event::default().json_data(psi_res) {
                    Ok(event) => event,
                    Err(_) => {
                        eprintln!("Error creating event from PSI report data for: {}", url);
                        break;
                    }
                };

                if sender.send(Ok(psi_event)).await.is_err() {
                    break;
                }

                sleep(Duration::from_secs(2)).await;
            }
        });
    }

    Ok(())
}
