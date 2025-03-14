use anyhow::{Context, Result};
use axum::{routing::get, Router};
use dotenv::dotenv;
use sitemaps::extract_sitemap_url_list;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod client;
mod handlers;
mod sitemaps;

use handlers::sse_reports_hanlder;

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
    // tracing_subscriber::fmt::init();
    tracing_subscriber::registry().with(fmt::layer()).init();
    let app = Router::new()
        .route("/reports", get(sse_reports_hanlder))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind(server_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();

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
