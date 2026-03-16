// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Web API for getting more than 1 tag at a time
//!

use crate::ApiError;
use axum::{Json, extract::State};
use bool_tag_expr::Tags;
use open_timeline_crud::FetchAll;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

// TODO: split the entity tags and timeline tags
/// Get a list of tags
pub async fn handle_get_tags(
    State(pool): State<Arc<Pool<Sqlite>>>,
) -> Result<Json<Tags>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    Ok(Json(Tags::fetch_all(&mut transaction).await?))
}
