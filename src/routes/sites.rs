use axum::{extract::Path, http::StatusCode, Extension, Json};
use axum_macros::debug_handler;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
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
) -> Result<Json<sites::Model>, StatusCode> {
    let new_site = sites::ActiveModel {
        domain: sea_orm::ActiveValue::Set(payload.domain),
        ..Default::default()
    };

    let saved_site = new_site
        .insert(app_state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(saved_site))
}

pub async fn get_sites(
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Json<Vec<sites::Model>>, StatusCode> {
    let sites: Vec<sites::Model> = Sites::find()
        .all(app_state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(sites))
}

pub async fn get_site(
    Path(site_id): Path<i32>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Json<sites::Model>, StatusCode> {
    let site: sites::Model = Sites::find_by_id(site_id)
        .one(app_state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(site))
}

#[derive(Deserialize)]
pub struct UpdateSite {
    pub domain: Option<String>,
}

pub async fn update_site(
    Path(site_id): Path<i32>,
    Extension(app_state): Extension<Arc<AppState>>,
    Json(payload): Json<UpdateSite>,
) -> Result<Json<sites::Model>, StatusCode> {
    let site: sites::Model = Sites::find_by_id(site_id)
        .one(app_state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut site: sites::ActiveModel = site.into();

    if let Some(domain) = payload.domain {
        site.domain = Set(domain);
    }

    let site: sites::Model = site.update(app_state.db.as_ref()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(site))
}
