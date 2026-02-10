// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All of the central panels shown in the primary OpenTimeline GUI window
//! (not including games)
//!

mod app_info;
mod backup_merge_restore;
mod config;
mod databse_stats;
mod entity_counts;
mod search;
mod tag_counts;
mod timeline_counts;

pub use app_info::*;
pub use backup_merge_restore::*;
pub use config::*;
pub use databse_stats::*;
pub use entity_counts::*;
pub use search::*;
pub use tag_counts::*;
pub use timeline_counts::*;
