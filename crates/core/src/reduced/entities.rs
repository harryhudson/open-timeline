// SPDX-License-Identifier: MIT

//!
//! Collection of reduced entities
//!

use crate::{IsReducedCollection, IsReducedType, Name, OpenTimelineId, ReducedEntity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Container for a set of [`ReducedEntity`]s
#[rustfmt::skip]
#[derive(derive_more::IntoIterator, Serialize, Deserialize, Default, PartialEq, Eq, Debug, Clone, PartialOrd, Ord, Hash)]
#[into_iterator(owned, ref, ref_mut)]
pub struct ReducedEntities(BTreeSet<ReducedEntity>);

impl IsReducedCollection for ReducedEntities {
    type Item = ReducedEntity;

    fn collection(&self) -> &BTreeSet<<Self as IsReducedCollection>::Item> {
        &self.0
    }

    fn collection_mut(&mut self) -> &mut BTreeSet<<Self as IsReducedCollection>::Item> {
        &mut self.0
    }
}

// TODO: these are nearly the same as those for ReducedTimelines
impl FromIterator<ReducedEntity> for ReducedEntities {
    fn from_iter<T: IntoIterator<Item = ReducedEntity>>(iter: T) -> Self {
        ReducedEntities(iter.into_iter().collect())
    }
}

impl ReducedEntities {
    /// Create a new `ReducedEntities`
    pub fn new() -> Self {
        ReducedEntities(BTreeSet::new())
    }

    pub fn ordered_by_name(&self) -> Vec<ReducedEntity> {
        let mut sorted: Vec<_> = self.0.clone().into_iter().collect();
        sorted.sort_by(|a, b| a.name().as_str().cmp(b.name().as_str()));
        sorted
    }

    pub fn ordered_by_id(&self) -> Vec<ReducedEntity> {
        let mut sorted: Vec<_> = self.0.clone().into_iter().collect();
        sorted.sort_by_key(|a| a.id());
        sorted
    }

    pub fn ids(&self) -> BTreeSet<OpenTimelineId> {
        self.0.clone().into_iter().map(|e| e.id()).collect()
    }

    pub fn names(&self) -> BTreeSet<Name> {
        self.0
            .clone()
            .into_iter()
            .map(|e| e.name().to_owned())
            .collect()
    }
}
