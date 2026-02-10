// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Timeline entity counts
//!

use crate::{CrudError, FetchById, SortAlphabetically, SortByNumber};
use open_timeline_core::{HasIdAndName, OpenTimelineId, TimelineEdit, TimelineView};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};

/// Holds a timeline and the number of entities it has
#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimelineCount {
    timeline: TimelineEdit,
    entity_count: i64,
}

impl TimelineCount {
    pub fn timeline(&self) -> &TimelineEdit {
        &self.timeline
    }

    pub fn entity_count(&self) -> &i64 {
        &self.entity_count
    }
}

/// A collection of [`TimelineCount`]
#[derive(
    derive_more::IntoIterator,
    derive_more::Index,
    Clone,
    Debug,
    Deserialize,
    Serialize,
    Hash,
    PartialEq,
    Eq,
)]
#[into_iterator(owned, ref, ref_mut)]
pub struct TimelineCounts(Vec<TimelineCount>);

impl FromIterator<TimelineCount> for TimelineCounts {
    fn from_iter<I: IntoIterator<Item = TimelineCount>>(iter: I) -> Self {
        TimelineCounts(iter.into_iter().collect())
    }
}

impl TimelineCounts {
    /// Get all unique tags (entity & timeline) in the database
    pub async fn fetch_all(transaction: &mut Transaction<'_, Sqlite>) -> Result<Self, CrudError> {
        // Get all timeline IDs
        let timeline_ids = sqlx::query_scalar!(
            r#"
                SELECT id AS "id: OpenTimelineId"
                FROM timelines
            "#
        )
        .fetch_all(&mut **transaction)
        .await?;

        let mut timeline_counts = Vec::new();

        for timeline_id in timeline_ids {
            // TODO: this feels silly
            // Get the total entity count
            let timeline = TimelineView::fetch_by_id(transaction, &timeline_id).await?;
            let entity_count = timeline
                .entities()
                .clone()
                .map_or(0, |entities| entities.len());

            // Get the timeline edit
            let timeline_edit = TimelineEdit::fetch_by_id(transaction, &timeline_id).await?;

            // Push to the collection
            timeline_counts.push(TimelineCount {
                timeline: timeline_edit,
                entity_count: entity_count as i64,
            });
        }

        Ok(TimelineCounts(timeline_counts))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn sort_by_count(&mut self, order: &SortByNumber) {
        match order {
            SortByNumber::Ascending => self
                .0
                .sort_by_key(|item| std::cmp::Reverse(item.entity_count)),
            SortByNumber::Descending => self.0.sort_by_key(|item| item.entity_count),
        }
    }

    pub fn sort_by_name(&mut self, order: &SortAlphabetically) {
        match order {
            SortAlphabetically::AToZ => self
                .0
                .sort_by(|a, b| a.timeline.name().cmp(b.timeline.name())),
            SortAlphabetically::ZToA => self
                .0
                .sort_by(|a, b| b.timeline.name().cmp(a.timeline.name())),
        }
    }
}
