use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    routing::{get, post},
    Extension, Router,
};
use dotenv::dotenv;
use sea_orm::{Database, DatabaseConnection};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod client;
pub mod config;
mod entities;
mod routes;
mod utils;

use entities::{prelude::*, *};
use routes::{reports, sites};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let _psi_url = std::env::var("PSI_URL").context("PSI_URL not found")?;
    let _psi_key = std::env::var("PSI_KEY").context("PSI_KEY not found")?;
    let server_url = std::env::var("SERVER_URL").context("SERVER_URL not found")?;
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL not found")?;

    let db: Arc<DatabaseConnection> = Arc::new(Database::connect(database_url).await?);
    let app_state = Arc::new(AppState { db });

    tracing_subscriber::registry().with(fmt::layer()).init();

    let app = Router::new()
        .route("/reports", get(reports::sse_reports_handler))
        .route("/sites", post(sites::create_site_handler))
        .route("/sites", get(sites::get_sites))
        .route("/sites/{site_id}", get(sites::get_site))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(Extension(app_state.clone()));

    let listener = tokio::net::TcpListener::bind(server_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
