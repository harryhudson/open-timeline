// SPDX-License-Identifier: MIT

//!
//! Collection of reduced timelines
//!

use crate::{IsReducedCollection, IsReducedType, Name, OpenTimelineId, ReducedTimeline};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Container for a set of [`ReducedTimeline`]s
#[rustfmt::skip]
#[derive(derive_more::IntoIterator, Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, PartialOrd, Ord, Hash)]
#[into_iterator(owned, ref, ref_mut)]
pub struct ReducedTimelines(BTreeSet<ReducedTimeline>);

impl IsReducedCollection for ReducedTimelines {
    type Item = ReducedTimeline;

    fn collection(&self) -> &BTreeSet<<Self as IsReducedCollection>::Item> {
        &self.0
    }

    fn collection_mut(&mut self) -> &mut BTreeSet<<Self as IsReducedCollection>::Item> {
        &mut self.0
    }
}

// TODO: these are nearly the same as those for ReducedEntities
impl FromIterator<ReducedTimeline> for ReducedTimelines {
    fn from_iter<T: IntoIterator<Item = ReducedTimeline>>(iter: T) -> Self {
        ReducedTimelines(iter.into_iter().collect())
    }
}

impl ReducedTimelines {
    pub fn new() -> Self {
        ReducedTimelines(BTreeSet::new())
    }

    // TODO: trait? (also in reduced_entities.rs)
    pub fn ordered_by_name(&self) -> Vec<ReducedTimeline> {
        let mut sorted: Vec<_> = self.0.clone().into_iter().collect();
        sorted.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));
        sorted
    }

    // TODO: trait? (also in reduced_entities.rs)
    pub fn ordered_by_id(&self) -> Vec<ReducedTimeline> {
        let mut sorted: Vec<_> = self.0.clone().into_iter().collect();
        sorted.sort_by_key(|a| a.id());
        sorted
    }

    pub fn ids(&self) -> BTreeSet<OpenTimelineId> {
        self.0
            .clone()
            .into_iter()
            .map(|timeline| timeline.id())
            .collect()
    }

    pub fn names(&self) -> BTreeSet<Name> {
        self.0
            .clone()
            .into_iter()
            .map(|timeline| timeline.name().to_owned())
            .collect()
    }
}
