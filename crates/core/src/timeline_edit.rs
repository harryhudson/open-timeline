// SPDX-License-Identifier: MIT

//!
//! The OpenTimeline timeline view type
//!

use crate::{HasIdAndName, Name, OpenTimelineId, ReducedEntities, ReducedTimelines};
use bool_tag_expr::{BoolTagExpr, Tag, Tags};
use serde::{Deserialize, Serialize};

/// Represents the information needed for creating and updating a timeline
///
/// This is the datastructure used to backup & restore timelines
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimelineEdit {
    /// The timeline's ID
    id: Option<OpenTimelineId>,

    /// The timeline's name
    name: Name,

    /// The timeline's direct `Entity`s
    bool_expr: Option<BoolTagExpr>,

    /// The timeline's direct `Entity`s
    entities: Option<ReducedEntities>,

    /// The timeline's direct subtimelines
    subtimelines: Option<ReducedTimelines>,

    /// The timeline's tags
    tags: Option<Tags>,
}

impl TimelineEdit {
    /// Create a new [`TimelineEdit`]
    pub fn from(
        id: Option<OpenTimelineId>,
        name: Name,
        bool_expr: Option<BoolTagExpr>,
        entities: Option<ReducedEntities>,
        subtimelines: Option<ReducedTimelines>,
        tags: Option<Tags>,
    ) -> Result<TimelineEdit, ()> {
        let mut timeline = TimelineEdit {
            id,
            name,
            bool_expr,
            entities: None,
            subtimelines: None,
            tags: None,
        };

        // TODO: some validation?
        timeline.entities = entities;
        timeline.subtimelines = subtimelines;
        timeline.tags = tags;
        Ok(timeline)
    }

    /// Clear the timeline's ID
    pub fn clear_id(&mut self) {
        self.id = None;
    }

    /// Borrow the timeline's boolean tag expr
    pub fn bool_expr(&self) -> &Option<BoolTagExpr> {
        &self.bool_expr
    }

    /// Clear the timeline's boolean tag expr
    pub fn clear_bool_expr(&mut self) {
        self.bool_expr = None;
    }

    /// Borrow the timeline's entities
    pub fn entities(&self) -> &Option<ReducedEntities> {
        &self.entities
    }

    /// Clear the timeline's entities
    pub fn clear_entities(&mut self) {
        self.entities = None;
    }

    /// Borrow the timeline's subtimelines
    pub fn subtimelines(&self) -> &Option<ReducedTimelines> {
        &self.subtimelines
    }

    /// Clear the timeline's subtimelines
    pub fn clear_subtimelines(&mut self) {
        self.subtimelines = None;
    }

    /// Borrow the timeline's [`Tags`]
    pub fn tags(&self) -> &Option<Tags> {
        &self.tags
    }

    /// Mutably borrow the timeline's [`Tags`]
    pub fn tags_mut(&mut self) -> &mut Option<Tags> {
        &mut self.tags
    }

    /// Add a tag to the timeline
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.get_or_insert_with(Tags::new).insert(tag);
    }

    /// Remove a tag from the timeline
    pub fn remove_tag(&mut self, tag: &Tag) {
        if let Some(tags) = self.tags.as_mut() {
            tags.remove(tag);
            if tags.is_empty() {
                self.tags = None
            }
        }
    }

    /// Set the timeline's [`Tags`]
    pub fn set_tags(&mut self, tags: Tags) {
        self.tags = (!tags.is_empty()).then_some(tags);
    }

    /// Clear the timeline's [`Tags`] and set to `None`
    pub fn clear_tags(&mut self) {
        self.tags = None;
    }
}

impl HasIdAndName for TimelineEdit {
    fn id(&self) -> Option<OpenTimelineId> {
        self.id
    }

    fn set_id(&mut self, id: OpenTimelineId) {
        self.id = Some(id)
    }

    fn name(&self) -> &Name {
        &self.name
    }

    fn set_name(&mut self, name: Name) {
        self.name = name
    }
}
