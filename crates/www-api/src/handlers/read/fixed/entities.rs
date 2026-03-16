// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Static Web API for fetching more than 1 entity at a time
//!

use crate::ApiError;
use axum::Json;
use axum::extract::State;
use open_timeline_core::{Entity, IsReducedType, ReducedEntities};
use open_timeline_crud::{FetchAll, FetchById};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

/// Handle a request to fetch all [`ReducedEntities`]
pub async fn handle_get_entities_reduced(
    State(pool): State<Arc<Pool<Sqlite>>>,
) -> Result<Json<ReducedEntities>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    Ok(Json(ReducedEntities::fetch_all(&mut transaction).await?))
}

/// Handle a request to fetch all [`Entity`]s
pub async fn handle_get_entities_full(
    State(pool): State<Arc<Pool<Sqlite>>>,
) -> Result<Json<Vec<Entity>>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    let mut full = Vec::new();
    for reduced in ReducedEntities::fetch_all(&mut transaction).await? {
        full.push(Entity::fetch_by_id(&mut transaction, &reduced.id()).await?);
    }
    Ok(Json(full))
}
