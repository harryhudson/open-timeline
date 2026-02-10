// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All requests that aren't GET requests
//!

pub mod entity;
pub mod timeline;

use axum::{
    Router,
    routing::{patch, put},
};
pub use entity::*;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
pub use timeline::*;

///
pub fn router() -> Result<Router<Arc<Pool<Sqlite>>>, sqlx::Error> {
    // Non-GET request routes for API v1
    #[rustfmt::skip]
    let apiv1 = Router::new()
        .route("/entity",                                    put(handle_put_entity))
        .route("/entity/{id-or-name}",                       patch(handle_patch_entity)
                                                                                .delete(handle_delete_entity))
        .route("/timeline",                                  put(handle_put_timeline))
        .route("/timeline/{id-or-name}",                     patch(handle_patch_timeline)
                                                                                .delete(handle_delete_timeline))
        .route("/timeline/{id-or-name}/entity/{id-or-name}", put(handle_put_timeline_entity)
                                                                                .delete(handle_delete_timeline_entity));

    Ok(apiv1)
}
