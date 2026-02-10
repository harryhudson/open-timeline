// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Timeline entity counts
//!

use crate::{CrudError, FetchById, SortAlphabetically, SortByNumber};
use log::info;
use open_timeline_core::{Date, Entity, HasIdAndName, Name, OpenTimelineId};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};

/// Holds a timeline and the number of entities it has
#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct EntityCount {
    /// The entity's ID
    id: OpenTimelineId,

    /// The entity's name
    name: Name,

    /// When did the entity begin/start
    start: Date,

    /// When did the entity end/finish (if it has)
    end: Option<Date>,

    /// Tags count for the entity
    tag_count: usize,
}

impl EntityCount {
    pub fn id(&self) -> OpenTimelineId {
        self.id
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn start(&self) -> Date {
        self.start
    }

    pub fn end(&self) -> Option<Date> {
        self.end
    }

    pub fn tag_count(&self) -> usize {
        self.tag_count
    }
}

/// A collection of [`EntityCount`]
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
pub struct EntityCounts(Vec<EntityCount>);

impl FromIterator<EntityCount> for EntityCounts {
    fn from_iter<I: IntoIterator<Item = EntityCount>>(iter: I) -> Self {
        EntityCounts(iter.into_iter().collect())
    }
}

impl EntityCounts {
    /// Get all unique tags (entity & timeline) in the database
    pub async fn fetch_all(transaction: &mut Transaction<'_, Sqlite>) -> Result<Self, CrudError> {
        // Get all timeline IDs
        let entity_ids = sqlx::query_scalar!(
            r#"
                SELECT id AS "id: OpenTimelineId"
                FROM entities
            "#
        )
        .fetch_all(&mut **transaction)
        .await?;

        let mut entities = Vec::new();

        for entity_id in entity_ids {
            entities.push(Entity::fetch_by_id(transaction, &entity_id).await?);
        }

        let entity_counts = entities
            .iter()
            .map(|entity| EntityCount {
                id: entity.id().unwrap(),
                name: entity.name().clone(),
                start: entity.start(),
                end: entity.end(),
                tag_count: entity
                    .tags()
                    .as_ref()
                    .map(|tags| tags.len())
                    .unwrap_or_default(),
            })
            .collect();

        Ok(entity_counts)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn sort_by_name(&mut self, order: &SortAlphabetically) {
        match order {
            SortAlphabetically::AToZ => self.0.sort_by(|a, b| a.name().cmp(&b.name())),
            SortAlphabetically::ZToA => self.0.sort_by(|a, b| b.name().cmp(&a.name())),
        }
    }

    pub fn sort_by_start_date(&mut self, order: &SortByNumber) {
        info!("Sorting entities by start date");
        match order {
            SortByNumber::Ascending => self.0.sort_by(|a, b| b.start().cmp(&a.start())),
            SortByNumber::Descending => self.0.sort_by(|a, b| a.start().cmp(&b.start())),
        }
    }

    pub fn sort_by_end_date(&mut self, order: &SortByNumber) {
        match order {
            SortByNumber::Ascending => self.0.sort_by(|a, b| b.end().cmp(&a.end())),
            SortByNumber::Descending => self.0.sort_by(|a, b| a.end().cmp(&b.end())),
        }
    }

    pub fn sort_by_tag_count(&mut self, order: &SortByNumber) {
        match order {
            SortByNumber::Ascending => self.0.sort_by_key(|e| std::cmp::Reverse(e.tag_count())),
            SortByNumber::Descending => self.0.sort_by_key(|e| e.tag_count()),
        }
    }
}
