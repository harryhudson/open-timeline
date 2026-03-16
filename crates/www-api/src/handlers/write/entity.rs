// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Web API for a single entity
//!

use crate::{ApiError, helpers::*};
use axum::Json;
use axum::extract::{Path, State};
use open_timeline_core::Entity;
use open_timeline_crud::DeleteById;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

/// Handle a request to create an entity
pub async fn handle_put_entity(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Json(mut payload): Json<Entity>,
) -> Result<Json<Entity>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();

    // TODO: move this into macro (was having difficulty)
    payload.clear_id();
    let result = save_new(&mut transaction, payload).await?;
    transaction.commit().await?;
    Ok(result)
}

/// Handle a request to update an entity
pub async fn handle_patch_entity(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Json(payload): Json<Entity>,
) -> Result<Json<Entity>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    let result = patch(&mut transaction, payload).await?;
    transaction.commit().await?;
    Ok(result)
}

/// Handle a request to delete an entity
pub async fn handle_delete_entity(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Path(id_or_name): Path<String>,
) -> Result<Json<()>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    let id = entity_id_from_id_or_name(&mut transaction, id_or_name).await?;
    Entity::delete_by_id(&mut transaction, &id).await?;
    transaction.commit().await?;
    // TODO: correct? Or wanted?
    Ok(Json(()))
}
