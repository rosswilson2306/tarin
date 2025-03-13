use std::{convert::Infallible, time::Duration};

use anyhow::{Context, Result};
use axum::{
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use dotenv::dotenv;
use sitemaps::extract_sitemap_url_list;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::{fs::File, time::interval};
use tokio_stream::StreamExt as _;

mod client;
mod sitemaps;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let _psi_url = std::env::var("PSI_URL").context("PSI_URL not found")?;
    let _psi_key = std::env::var("PSI_KEY").context("PSI_KEY not found")?;
    let server_url = std::env::var("SERVER_URL").context("SERVER_URL not found")?;

    let websites = get_base_sites("sites.txt").await?;

    for url in websites.iter() {
        extract_sitemap_url_list(url).await?;
    }

    // TODO: look into logging format
    tracing_subscriber::fmt::init();
    let app = Router::new().route("/reports", get(sse_reports_hanlder));

    let listener = tokio::net::TcpListener::bind(server_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn sse_reports_hanlder() -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::wrappers::IntervalStream::new(interval(Duration::from_secs(2)))
        .map(|_| Ok(Event::default().data("New report available")));

    Sse::new(stream).keep_alive(KeepAlive::default())
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
