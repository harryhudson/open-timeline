// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All GET requests
//!

use crate::ApiMode;
use axum::{Router, routing::get};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

mod dynamic;
mod fixed;

///
pub fn router(api_mode: ApiMode) -> Result<Router<Arc<Pool<Sqlite>>>, sqlx::Error> {
    // GET request routes for API v1
    #[rustfmt::skip]
    let apiv1 = Router::new()
        .route("/entity/{id-or-name}",           get(fixed::entity::handle_get_entity))
        .route("/entity/{id-or-name}/timelines", get(fixed::entity::handle_get_entity_direct_member_of_which_timelines))
        .route("/timeline/{id-or-name}/edit",    get(fixed::timeline::handle_get_timeline_for_edit))
        .route("/timeline/{id-or-name}/view",    get(fixed::timeline::handle_get_timeline_for_view))
        .route("/tags",                          get(fixed::tags::handle_get_tags));

    let apiv1 = match api_mode {
        ApiMode::Static => {
            #[rustfmt::skip]
            let apiv1 = apiv1
                .route("/entities/reduced",      get(fixed::entities::handle_get_entities_reduced))
                .route("/entities/full",         get(fixed::entities::handle_get_entities_full))
                .route("/timelines/reduced",     get(fixed::timelines::handle_get_timelines_reduced))
                .route("/timelines/edit",        get(fixed::timelines::handle_get_timelines_edit));
            apiv1
        }
        ApiMode::Dynamic => {
            #[rustfmt::skip]
            let apiv1 = apiv1
                .route("/entities/reduced",      get(dynamic::entities::handle_get_entities_reduced))
                .route("/timelines/reduced",     get(dynamic::timelines::handle_get_timelines_reduced))
                .route("/entities/random",       get(dynamic::entities::handle_get_random_entities))
                .route("/timelines/random",      get(dynamic::timelines::handle_get_random_timelines));
            apiv1
        }
    };

    Ok(apiv1)
}
