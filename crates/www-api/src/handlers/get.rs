// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All GET requests
//!

use crate::ApiMode;
use axum::{Router, routing::get};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

mod dynamic;
mod non_dynamic;

///
pub fn router(api_mode: ApiMode) -> Result<Router<Arc<Pool<Sqlite>>>, sqlx::Error> {
    // GET request routes for API v1
    #[rustfmt::skip]
    let apiv1 = Router::new()
        .route("/entity/{id-or-name}",           get(non_dynamic::entity::handle_get_entity))
        .route("/entity/{id-or-name}/timelines", get(non_dynamic::entity::handle_get_entity_direct_member_of_which_timelines))
        .route("/timeline/{id-or-name}/meta",    get(non_dynamic::timeline::handle_get_timeline_for_edit))
        .route("/timeline/{id-or-name}/all",     get(non_dynamic::timeline::handle_get_timeline_for_view))
        .route("/tags",                          get(non_dynamic::tags::handle_get_tags));

    let apiv1 = match api_mode {
        ApiMode::Static => {
            #[rustfmt::skip]
            let apiv1 = apiv1
                .route("/entities", get(non_dynamic::entities::handle_get_entities))
                .route("/timelines",get(non_dynamic::timelines::handle_get_timelines));
            apiv1
        }
        ApiMode::Dynamic => {
            #[rustfmt::skip]
            let apiv1 = apiv1
                .route("/entities",         get(dynamic::entities::handle_get_entities))
                .route("/timelines",        get(dynamic::timelines::handle_get_timelines))
                .route("/entities/random",  get(dynamic::entities::handle_get_random_entities))
                .route("/timelines/random", get(dynamic::timelines::handle_get_random_timelines));
            apiv1
        }
    };

    Ok(apiv1)
}
