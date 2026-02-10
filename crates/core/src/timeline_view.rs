// SPDX-License-Identifier: MIT

//!
//! The OpenTimeline timeline view type
//!

use crate::{Entity, HasIdAndName, Name, OpenTimelineId};
use serde::Serialize;

/// Holds the information needed to draw a timeline
///
/// See also [`crate::TimelineEdit`]
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct TimelineView {
    /// The timeline's ID
    id: OpenTimelineId,

    /// The timeline's name
    name: Name,

    /// All the [`Entity`]s that are part of the timeline directly as well as
    /// all of its subtimeline
    entities: Option<Vec<Entity>>,
}

impl TimelineView {
    /// Create a TimelineView
    pub fn from(id: OpenTimelineId, name: Name, entities: Option<Vec<Entity>>) -> Self {
        Self { id, name, entities }
    }

    /// Borrow the timeline's ID
    pub fn id(&self) -> &OpenTimelineId {
        &self.id
    }

    /// Borrow the timeline's name
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Borrow the timeline's entities
    pub fn entities(&self) -> &Option<Vec<Entity>> {
        &self.entities
    }
}

impl HasIdAndName for TimelineView {
    fn id(&self) -> Option<OpenTimelineId> {
        Some(self.id)
    }
    fn set_id(&mut self, id: OpenTimelineId) {
        self.id = id
    }
    fn name(&self) -> &Name {
        &self.name
    }
    fn set_name(&mut self, name: Name) {
        self.name = name
    }
}
