// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Web API for a single timeline
//!

use crate::{ApiError, helpers::*};
use axum::Json;
use axum::extract::{Path, State};
use open_timeline_core::TimelineEdit;
use open_timeline_crud::{CrudError, DeleteById, DeleteByName, IdOrName};
use open_timeline_crud::{
    delete_timeline_entity, entity_id_from_name, entity_id_or_name, insert_timeline_entity,
    timeline_id_from_name, timeline_id_or_name,
};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

// NOTE: if input has "sbtmlines" (spelt incorrectly) it won't throw an error becuase it's Option<>al
// do stuff with input
/// Handle a request to create a timeline
pub async fn handle_put_timeline(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Json(mut payload): Json<TimelineEdit>,
) -> Result<Json<TimelineEdit>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();

    // TODO: correct? What if the ID is set and already exists? Should error?
    payload.clear_id();

    let result = save_new(&mut transaction, payload).await?;
    transaction.commit().await?;
    Ok(result)
}

/// Handle a request to update a timeline
pub async fn handle_patch_timeline(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Json(payload): Json<TimelineEdit>,
) -> Result<Json<TimelineEdit>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    let result = patch(&mut transaction, payload).await?;
    transaction.commit().await?;
    Ok(result)
}

/// Handle a request to delete a timeline
pub async fn handle_delete_timeline(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Path(id_or_name): Path<String>,
) -> Result<Json<()>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    match timeline_id_or_name(&mut transaction, id_or_name).await? {
        Some(IdOrName::Id(id)) => Ok(TimelineEdit::delete_by_id(&mut transaction, &id).await?),
        Some(IdOrName::Name(name)) => {
            Ok(TimelineEdit::delete_by_name(&mut transaction, &name).await?)
        }
        None => Err(CrudError::NotInDb),
    }?;
    transaction.commit().await?;
    Ok(Json(()))
}

/// Handle a request to add an entity to a timeline
pub async fn handle_put_timeline_entity(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Path(timeline_id_or_name_str): Path<String>,
    Path(entity_id_or_name_str): Path<String>,
) -> Result<Json<()>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();

    let timeline_id = match timeline_id_or_name(&mut transaction, timeline_id_or_name_str).await? {
        Some(IdOrName::Id(id)) => id,
        Some(IdOrName::Name(name)) => timeline_id_from_name(&mut transaction, &name).await?,
        None => Err(CrudError::TimelineNotInDb)?,
    };

    let entity_id = match entity_id_or_name(&mut transaction, entity_id_or_name_str).await? {
        Some(IdOrName::Id(id)) => id,
        Some(IdOrName::Name(name)) => entity_id_from_name(&mut transaction, &name).await?,
        None => Err(CrudError::EntityNotInDb)?,
    };

    insert_timeline_entity(&mut transaction, &timeline_id, &entity_id).await?;

    transaction.commit().await?;
    Ok(Json(()))
}

/// Handle a request to delete an entity from a timeline
pub async fn handle_delete_timeline_entity(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Path(timeline_id_or_name_str): Path<String>,
    Path(entity_id_or_name_str): Path<String>,
) -> Result<Json<()>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();

    let timeline_id = match timeline_id_or_name(&mut transaction, timeline_id_or_name_str).await? {
        Some(IdOrName::Id(id)) => id,
        Some(IdOrName::Name(name)) => timeline_id_from_name(&mut transaction, &name).await?,
        None => Err(CrudError::TimelineNotInDb)?,
    };

    let entity_id = match entity_id_or_name(&mut transaction, entity_id_or_name_str).await? {
        Some(IdOrName::Id(id)) => id,
        Some(IdOrName::Name(name)) => entity_id_from_name(&mut transaction, &name).await?,
        None => Err(CrudError::EntityNotInDb)?,
    };

    delete_timeline_entity(&mut transaction, &timeline_id, &entity_id).await?;

    transaction.commit().await?;
    Ok(Json(()))
}
