use axum::{extract::Path, Extension, Json};
use sea_orm::{ActiveModelTrait, EntityTrait};
use serde::Deserialize;
use std::sync::Arc;
// use axum_macros::debug_handler;

use crate::{
    entities::sites::{self, Entity as Sites},
    AppState,
};

#[derive(Deserialize)]
pub struct NewSite {
    pub domain: String,
}

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

pub async fn get_sites(
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<sites::Model>>, axum::http::StatusCode> {
    let sites: Vec<sites::Model> = Sites::find()
        .all(app_state.db.as_ref())
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(sites))
}

pub async fn get_site(
    Path(site_id): Path<i32>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Json<sites::Model>, axum::http::StatusCode> {
    let site: Option<sites::Model> = Sites::find_by_id(site_id)
        .one(app_state.db.as_ref())
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let site = match site {
        Some(s) => s,
        None => return Err(axum::http::StatusCode::NOT_FOUND)
    };

    Ok(Json(site))
}
