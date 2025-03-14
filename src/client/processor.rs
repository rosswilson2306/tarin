use anyhow::Result;
use axum::response::sse::Event;
use std::{convert::Infallible, time::Duration};
use tokio::{sync::mpsc, task, time::sleep};

use crate::{client::sitemaps::extract_sitemap_url_list, utils::get_base_sites};

pub async fn process_websites(
    sender: mpsc::Sender<std::result::Result<Event, Infallible>>,
) -> Result<()> {
    let websites = get_base_sites("sites.txt").await?;

    for site in websites {
        let sender = sender.clone();
        task::spawn(async move {
            if let Ok(site_urls) = extract_sitemap_url_list(&site).await {
                for url in site_urls {
                    // TODO: fetch report for url
                    if sender.send(Ok(Event::default().data(url))).await.is_err() {
                        break;
                    }
                    sleep(Duration::from_secs(2)).await;
                }
            } else {
                eprintln!("Failed to extract urls from sitemaps")
            }
        });
    }

    todo!()
}
