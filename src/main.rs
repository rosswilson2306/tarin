use anyhow::{Context, Result};
use axum::{routing::get, Router};
use dotenv::dotenv;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod client;
mod handlers;
mod utils;
mod config;

use handlers::sse_reports_handler;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let _psi_url = std::env::var("PSI_URL").context("PSI_URL not found")?;
    let _psi_key = std::env::var("PSI_KEY").context("PSI_KEY not found")?;
    let server_url = std::env::var("SERVER_URL").context("SERVER_URL not found")?;

    // TODO: look into logging format
    tracing_subscriber::registry().with(fmt::layer()).init();
    let app = Router::new()
        .route("/reports", get(sse_reports_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind(server_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
