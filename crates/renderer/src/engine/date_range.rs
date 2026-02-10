// SPDX-License-Identifier: MIT

//!
//! Date range
//!

use open_timeline_core::Date;
use std::fmt::Debug;

// TODO: use the Year type instead of i32s(?)
/// The timeline's date ranges.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TimelineDateRange {
    /// The decade in which the timeline starts
    pub decade_range_start: i32,

    /// The decade before which the timeline ends
    pub decade_range_end: i32,

    /// The earliest entity start year
    pub earliest_year: i32,

    /// The latest entity end year (ignores those without end dates)
    pub latest_year: i32,

    /// Optional user-set start date cutoff.  If set and an entity starts before
    /// this, it isn't shown on the timeline
    pub start_date_cutoff: Option<Date>,

    /// Optional user-set end date cutoff.  If set and an entity ends after
    /// this, it isn't shown on the timeline
    pub end_date_cutoff: Option<Date>,

    /// The number of decades being shown on the timeline (not set by users)
    pub decade_count: i32,
}
