// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All query parameter structs
//!

use crate::DEFAULT_LIMIT_PARTIAL_NAME_QUERY;
use open_timeline_crud::Limit;
use serde::Deserialize;

// TODO: I think partial_name should be a `Name`
// Limit to ~32,000
/// Query parameters used when fetching by a partial name (string) with a limit
#[derive(Deserialize)]
pub struct PartialNameQueryParams {
    #[serde(rename = "partial-name")]
    pub partial_name: String,
    pub limit: Limit,
}

impl Default for PartialNameQueryParams {
    fn default() -> Self {
        PartialNameQueryParams {
            partial_name: String::from(""),
            limit: Limit(DEFAULT_LIMIT_PARTIAL_NAME_QUERY),
        }
    }
}
