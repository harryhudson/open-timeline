// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Generic helpers
//!

use crate::ApiError;
use axum::{Json, http::StatusCode};
use open_timeline_core::{HasIdAndName, OpenTimelineId};
use open_timeline_crud::{
    Create, FetchByName, IdOrName, Update, entity_id_from_name, entity_id_or_name,
};
use serde::Serialize;
use sqlx::{Sqlite, Transaction};

// TODO: keep in this form? or type alias?
#[derive(Serialize)]
pub struct ErrorMsg {
    pub error_msg: String,
}

// TODO: check
pub async fn save_new<T: Create + FetchByName + HasIdAndName>(
    transaction: &mut Transaction<'_, Sqlite>,
    mut thing_to_create: T,
) -> Result<Json<T>, ApiError> {
    thing_to_create.create(transaction).await?;
    let created = T::fetch_by_name(transaction, thing_to_create.name()).await?;
    Ok(Json(created))
}

// TODO: check
pub async fn patch<T: std::fmt::Debug + Update + FetchByName + HasIdAndName>(
    transaction: &mut Transaction<'_, Sqlite>,
    mut thing_to_patch: T,
) -> Result<Json<T>, ApiError> {
    thing_to_patch.update(transaction).await?;
    let updated = T::fetch_by_name(transaction, thing_to_patch.name()).await?;
    Ok(Json(updated))
}

// TODO: correct? could be simpler?
pub async fn entity_id_from_id_or_name(
    transaction: &mut Transaction<'_, Sqlite>,
    id_or_name: String,
) -> Result<OpenTimelineId, ApiError> {
    match entity_id_or_name(transaction, id_or_name).await? {
        Some(IdOrName::Id(id)) => Ok(id),
        Some(IdOrName::Name(name)) => Ok(entity_id_from_name(transaction, &name).await?),
        None => Err(ApiError((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMsg {
                // todo: if ID say failed to delete entity with ID X, likewise if name
                error_msg: "FAILED to fetch".to_string(),
            }),
        ))),
    }
}
