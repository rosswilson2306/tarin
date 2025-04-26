use axum::{Extension, Json};
use axum_macros::debug_handler;
use sea_orm::ActiveModelTrait;
use serde::Deserialize;
use std::sync::Arc;

use crate::{entities::sites as sites_entity, AppState};

#[derive(Deserialize)]
pub struct NewSite {
    pub domain: String,
}

#[debug_handler]
pub async fn create_site_handler(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(payload): Json<NewSite>,
) -> Result<Json<sites_entity::Model>, axum::http::StatusCode> {
    let new_site = sites_entity::ActiveModel {
        domain: sea_orm::ActiveValue::Set(payload.domain),
        ..Default::default()
    };

    let saved_site = new_site
        .insert(app_state.db.as_ref())
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(saved_site))
}
