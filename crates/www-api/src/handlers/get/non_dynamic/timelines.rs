// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Static Web API for fetching more than 1 entity at a time
//!

use crate::ApiError;
use axum::Json;
use axum::extract::State;
use open_timeline_core::ReducedTimelines;
use open_timeline_crud::FetchAll;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

/// Handle a request to fetch timelines whose name matches a partial name
pub async fn handle_get_timelines(
    State(pool): State<Arc<Pool<Sqlite>>>,
) -> Result<Json<ReducedTimelines>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    Ok(Json(ReducedTimelines::fetch_all(&mut transaction).await?))
}
