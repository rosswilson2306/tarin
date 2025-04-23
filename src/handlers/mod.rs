use axum::{
    response::sse::{Event, KeepAlive, Sse},
    Extension, Json,
};
use axum_macros::debug_handler;
use sea_orm::ActiveModelTrait;
use serde::Deserialize;
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{client::processor::process_websites, sites, AppState};

pub async fn sse_reports_handler() -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel(10);

    tokio::spawn(process_websites(tx));

    let stream = ReceiverStream::new(rx);
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(Deserialize)]
pub struct NewSite {
    pub domain: String,
}

#[debug_handler]
pub async fn create_site_handler(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(payload): Json<NewSite>,
) -> Result<Json<sites::Model>, axum::http::StatusCode> {
    let new_site = sites::ActiveModel {
        domain: sea_orm::ActiveValue::Set(payload.domain),
        ..Default::default()
    };

    let saved_site = new_site
        .insert(app_state.db.as_ref())
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(saved_site))
}
