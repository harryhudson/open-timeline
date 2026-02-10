// SPDX-License-Identifier: MIT

//!
//! Events
//!

use open_timeline_core::OpenTimelineId;
use serde::Serialize;
use std::fmt::Debug;

/// Interaction events
#[derive(Debug, Clone, Serialize)]
pub enum TimelineInteractionEvent {
    SingleClick(OpenTimelineId),
    DoubleClick(OpenTimelineId),
    TripleClick(OpenTimelineId),
    Hover(OpenTimelineId),
}
