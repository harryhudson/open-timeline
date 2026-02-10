// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! *Part of the wider OpenTimeline project*
//!
//! This crate provides the web API, which may also be run locally.  This means,
//! for example, that a company or group or individual could host their own
//! timeline API.  It also means that one can add entities and timelines to the
//! public OpenTimeline database by adding them locally, and them sending them
//! to OpenTimeline to be merged in.
//!

mod consts;
mod error;
mod handlers;
mod helpers;
mod queries;

use consts::*;
use error::*;
use queries::*;

use axum::Router;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{str::FromStr, sync::Arc};

/// API access mode (read-only or read-write)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiAccessMode {
    Read,
    ReadWrite,
}

/// API response mode (static or dynamic content)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiMode {
    Static,
    Dynamic,
}

/// Set up and serve the API
pub async fn prepare_api_router(
    db_url: &str,
    access_mode: ApiAccessMode,
    api_mode: ApiMode,
) -> Result<Router, sqlx::Error> {
    // TODO: test the read-only aspect?
    // Create connection options (whether the database is read-only or not)
    let connect_options =
        SqliteConnectOptions::from_str(db_url)?.read_only(access_mode == ApiAccessMode::Read);

    // Create a pool with those options
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Get the router
    let apiv1 = handlers::router(access_mode, api_mode)?;

    // Add the state
    let apiv1 = apiv1.with_state(Arc::new(pool));

    // Add URL path prefix
    let api = Router::new().nest("/api/v1", apiv1);

    // Return the router
    Ok(api)
}
