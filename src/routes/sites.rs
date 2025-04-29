use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DeleteResult, EntityTrait};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    entities::sites::{self, Entity as Sites},
    AppState,
};

#[derive(Deserialize)]
pub struct NewSite {
    pub domain: String,
}

pub async fn create_site_handler(
    State(app_state): State<Arc<AppState>>,
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
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<sites::Model>>, StatusCode> {
    let sites: Vec<sites::Model> = Sites::find()
        .all(app_state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(sites))
}

pub async fn get_site(
    Path(site_id): Path<i32>,
    State(app_state): State<Arc<AppState>>,
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
    State(app_state): State<Arc<AppState>>,
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

    let site: sites::Model = site
        .update(app_state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(site))
}

pub async fn delete_site(
    Path(site_id): Path<i32>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let result: DeleteResult = Sites::delete_by_id(site_id)
        .exec(app_state.db.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if result.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
