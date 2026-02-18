// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Web API for fetching more than 1 entity at a time
//!

use crate::helpers::ErrorMsg;
use crate::{
    ApiError, DEFAULT_LIMIT_RANDOM_ENTITIES, MAX_LIMIT_RANDOM_ENTITIES, PartialNameQueryParams,
};
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use open_timeline_core::{Entity, ReducedEntities};
use open_timeline_crud::{FetchByPartialName, Limit, fetch_random_entities};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;

/// Handle a request to fetch entities whose name matches a partial name
pub async fn handle_get_entities_reduced(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Query(params): Query<PartialNameQueryParams>,
) -> Result<Json<ReducedEntities>, ApiError> {
    // Get the transaction
    let mut transaction = pool.begin().await.unwrap();

    // TODO: is this correct?
    if params.partial_name.is_empty() {
        return Err(ApiError((
            StatusCode::BAD_REQUEST,
            Json(ErrorMsg {
                error_msg: "No 'partial-name' or empty in query param".to_string(),
            }),
        )));
    }

    Ok(Json(
        ReducedEntities::fetch_by_partial_name(&mut transaction, params.limit, "").await?,
    ))
}

// TODO: what query string is accepted? I think it's `limit=X`
/// Handle a request to fetch some random entities
pub async fn handle_get_random_entities(
    State(pool): State<Arc<Pool<Sqlite>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Entity>>, ApiError> {
    // Get the transaction
    let mut transaction = pool.begin().await.unwrap();

    let limit = params
        .get("limit")
        .and_then(|limit_str| limit_str.parse().ok())
        .map_or(Limit(DEFAULT_LIMIT_RANDOM_ENTITIES), |value: u32| {
            Limit(value.min(MAX_LIMIT_RANDOM_ENTITIES))
        });

    // TODO: proper error checking
    Ok(Json(fetch_random_entities(&mut transaction, limit).await?))
}
