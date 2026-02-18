// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Web API for fetching more than 1 timeline at a time
//!

use crate::{ApiError, PartialNameQueryParams, helpers::ErrorMsg};
use crate::{DEFAULT_LIMIT_RANDOM_TIMELINES, MAX_LIMIT_RANDOM_TIMELINES};
use axum::Json;
use axum::extract::Query;
use axum::{extract::State, http::StatusCode};
use open_timeline_core::ReducedTimelines;
use open_timeline_crud::{FetchByPartialName, Limit};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;

/// Handle a request to fetch timelines whose name matches a partial name
pub async fn handle_get_timelines_reduced(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Query(params): Query<PartialNameQueryParams>,
) -> Result<Json<ReducedTimelines>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();

    // TODO: should this be an error? or return all (with default limit?)
    if params.partial_name.is_empty() {
        return Err(ApiError((
            StatusCode::BAD_REQUEST,
            Json(ErrorMsg {
                error_msg: "No 'partial-name' or empty in query param".to_string(),
            }),
        )));
    }
    Ok(Json(
        ReducedTimelines::fetch_by_partial_name(
            &mut transaction,
            params.limit,
            &params.partial_name,
        )
        .await?,
    ))
}

// TODO split out into a fetch_random_timelines()
/// Handle a request to get some random timelines
pub async fn handle_get_random_timelines(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ReducedTimelines>, ApiError> {
    let mut transaction = pool.begin().await.unwrap();

    let limit = params
        .get("limit")
        .and_then(|limit_str| limit_str.parse().ok())
        .map_or(Limit(DEFAULT_LIMIT_RANDOM_TIMELINES), |value: u32| {
            Limit(value.min(MAX_LIMIT_RANDOM_TIMELINES))
        });

    Ok(Json(
        ReducedTimelines::fetch_by_partial_name(&mut transaction, limit, "").await?,
    ))
}
