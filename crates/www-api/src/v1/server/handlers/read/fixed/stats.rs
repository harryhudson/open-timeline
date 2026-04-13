// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Web API for getting dataset stats
//!

use crate::v1::error::ApiError;
use axum::{Json, extract::State};
use open_timeline_crud::DatabaseRowCount;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

/// Get stats
pub async fn handle_get_stats(
    State(pool): State<Arc<Pool<Sqlite>>>,
) -> Result<Json<DatabaseRowCount>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();
    Ok(Json(DatabaseRowCount::all(&mut transaction).await?))
}
