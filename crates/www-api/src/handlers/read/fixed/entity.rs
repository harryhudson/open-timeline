// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Web API for a single entity
//!

use crate::{ApiError, helpers::*};
use axum::Json;
use axum::extract::{Path, State};
use open_timeline_core::{Entity, ReducedTimelines};
use open_timeline_crud::{FetchById, fetch_timelines_that_entity_is_direct_member_of};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

/// Handle a request to fetch an entity
pub async fn handle_get_entity(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Path(id_or_name): Path<String>,
) -> Result<Json<Entity>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    let id = entity_id_from_id_or_name(&mut transaction, id_or_name).await?;
    let entity = Entity::fetch_by_id(&mut transaction, &id).await?;
    Ok(Json(entity))
}

/// Handle a request to delete an entity
pub async fn handle_get_entity_direct_member_of_which_timelines(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Path(id_or_name): Path<String>,
) -> Result<Json<ReducedTimelines>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    let id = entity_id_from_id_or_name(&mut transaction, id_or_name).await?;
    let result = fetch_timelines_that_entity_is_direct_member_of(&mut transaction, &id).await?;
    Ok(Json(result))
}
