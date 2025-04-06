use anyhow::{anyhow, Result};
use axum::response::sse::Event;
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, Semaphore},
    task,
    time::sleep,
};
use tracing::{error, info};

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
    let semaphore = Arc::new(Semaphore::new(10));

    for site in websites {
        let sender = sender.clone();
        let psi_client = psi_client.clone();
        let semaphore = semaphore.clone();

        task::spawn(async move {
            let _permit = match semaphore.acquire_owned().await {
                Ok(permit) => permit,
                Err(e) => return error!("Failed to acquire permit from semaphore: {e}"),
            };

            let site_urls = match extract_sitemap_url_list(&site).await {
                Ok(urls_list) => urls_list,
                Err(_) => {
                    return error!(
                        "Failed to extract urls from sitemaps for: {}",
                        site.as_ref()
                    )
                }
            };

            for url in site_urls {
                info!("Running PSI report for: {} ...", url);

                let psi_res = match psi_client.get_report(url.as_ref()).await {
                    Ok(res) => res,
                    Err(_) => {
                        error!("Error fetching report for: {}", url);
                        break;
                    }
                };

                let psi_event = match Event::default().json_data(psi_res) {
                    Ok(event) => event,
                    Err(_) => {
                        error!("Error creating event from PSI report data for: {}", url);
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
