// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All handlers
//!

use crate::{ApiAccessMode, ApiMode};
use axum::Router;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

pub mod read;
pub mod write;

/// Set up and serve the API
pub fn router(
    access_mode: ApiAccessMode,
    api_mode: ApiMode,
) -> Result<Router<Arc<Pool<Sqlite>>>, sqlx::Error> {
    // GET request routes for API v1
    let router = read::router(api_mode)?;

    // Non-GET request routes for API v1
    let router = match access_mode {
        ApiAccessMode::Read => router,
        ApiAccessMode::ReadWrite => router.merge(write::router()?),
    };

    Ok(router)
}
