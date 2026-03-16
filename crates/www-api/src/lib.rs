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

mod client;
mod consts;
mod error;
mod handlers;
mod helpers;
mod queries;

pub use client::*;

use consts::*;
use error::*;
use log::info;
use queries::*;

use axum::{Json, Router, routing::get};
use serde_json::json;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

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

// TODO: check pool is read-only if access mode and API mode are?
/// Helper for using the argue JSON web API
#[derive(Debug)]
pub struct OpenTimelineWebApi {}

impl OpenTimelineWebApi {
    /// Serve the JSON web APIv1
    pub async fn serve_v1(
        pool: Pool<Sqlite>,
        port: u16,
        access_mode: ApiAccessMode,
        api_mode: ApiMode,
    ) -> anyhow::Result<()> {
        // Get the router
        let api_router = prepare_api_router(pool, access_mode, api_mode)
            .await
            .unwrap();

        // Specify the IP addr and port number
        let addr = format!("0.0.0.0:{port}");

        // Bind the listener for new connections
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        // Print the address
        info!("http://{addr}");

        // Serve the server
        axum::serve(listener, api_router).await?;

        // Won't actually get here if the server is running
        Ok(())
    }
}

/// Set up and serve the API
async fn prepare_api_router(
    pool: Pool<Sqlite>,
    access_mode: ApiAccessMode,
    api_mode: ApiMode,
) -> anyhow::Result<Router> {
    // Get the router
    let apiv1 = handlers::router(access_mode, api_mode)?;

    // Add the state
    let apiv1 = apiv1.with_state(Arc::new(pool));

    // Add URL path prefix
    let api = Router::new().nest("/api/v1", apiv1);

    // Add /health endpoint
    let api = api.route("/health", get(|| async { Json(json!({ "status": "ok" })) }));

    // Return the router
    Ok(api)
}
