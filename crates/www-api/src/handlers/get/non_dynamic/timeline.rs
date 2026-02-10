// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Web API for a single timeline
//!

use crate::ApiError;
use axum::Json;
use axum::extract::{Path, State};
use open_timeline_core::{TimelineEdit, TimelineView};
use open_timeline_crud::{self, CrudError, FetchById, FetchByName, IdOrName, timeline_id_or_name};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

/// Handle a request to get a timeline for editing (i.e. a [`TimelineEdit`])
pub async fn handle_get_timeline_for_edit(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Path(id_or_name): Path<String>,
) -> Result<Json<TimelineEdit>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    let timeline = match timeline_id_or_name(&mut transaction, id_or_name).await? {
        Some(IdOrName::Id(id)) => Ok(TimelineEdit::fetch_by_id(&mut transaction, &id).await?),
        Some(IdOrName::Name(name)) => {
            Ok(TimelineEdit::fetch_by_name(&mut transaction, &name).await?)
        }
        None => Err(CrudError::NotInDb),
    }?;
    Ok(Json(timeline))
}

/// Handle a request to get a timeline for viewing (i.e. a [`TimelineView`])
pub async fn handle_get_timeline_for_view(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Path(id_or_name): Path<String>,
) -> Result<Json<TimelineView>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    Ok(Json(
        match timeline_id_or_name(&mut transaction, id_or_name).await? {
            Some(IdOrName::Id(id)) => Ok(TimelineView::fetch_by_id(&mut transaction, &id).await?),
            Some(IdOrName::Name(name)) => {
                Ok(TimelineView::fetch_by_name(&mut transaction, &name).await?)
            }
            None => Err(CrudError::NotInDb),
        }?,
    ))
}
