// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Static Web API for fetching more than 1 timeline at a time
//!

use crate::ApiError;
use axum::Json;
use axum::extract::State;
use open_timeline_core::{IsReducedType, ReducedTimelines, TimelineEdit};
use open_timeline_crud::{FetchAll, FetchById};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

/// Handle a request to fetch all [`ReducedTimelines`]
pub async fn handle_get_timelines_reduced(
    State(pool): State<Arc<Pool<Sqlite>>>,
) -> Result<Json<ReducedTimelines>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    Ok(Json(ReducedTimelines::fetch_all(&mut transaction).await?))
}

/// Handle a request to fetch all [`TimelineEdit`]s
pub async fn handle_get_timelines_edit(
    State(pool): State<Arc<Pool<Sqlite>>>,
) -> Result<Json<Vec<TimelineEdit>>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    let mut full = Vec::new();
    for reduced in ReducedTimelines::fetch_all(&mut transaction).await? {
        full.push(TimelineEdit::fetch_by_id(&mut transaction, &reduced.id()).await?);
    }
    Ok(Json(full))
}
