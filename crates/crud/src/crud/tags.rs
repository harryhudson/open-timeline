// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Tags
//!

use crate::{CrudError, FetchAll, RowsAffected, SortAlphabetically, SortByNumber};
use async_trait::async_trait;
use bool_tag_expr::{Tag, TagName, TagValue, Tags};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};

/// Holds a tag and the number of times it appears in the database
#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TagCount {
    tag: Tag,
    count: i64,
}

impl TagCount {
    pub fn tag(&self) -> &Tag {
        &self.tag
    }

    pub fn count(&self) -> &i64 {
        &self.count
    }
}

/// A collection of [`TagCount`]
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
pub struct TagCounts(Vec<TagCount>);

impl FromIterator<TagCount> for TagCounts {
    fn from_iter<I: IntoIterator<Item = TagCount>>(iter: I) -> Self {
        TagCounts(iter.into_iter().collect())
    }
}

impl TagCounts {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn sort_by_count(&mut self, order: &SortByNumber) {
        match order {
            SortByNumber::Ascending => self.0.sort_by_key(|item| std::cmp::Reverse(item.count)),
            SortByNumber::Descending => self.0.sort_by_key(|item| item.count),
        }
    }

    pub fn sort_by_tag_name(&mut self, order: &SortAlphabetically) {
        match order {
            SortAlphabetically::AToZ => self.0.sort_by(|a, b| a.tag.name.cmp(&b.tag.name)),
            SortAlphabetically::ZToA => self.0.sort_by(|a, b| b.tag.name.cmp(&a.tag.name)),
        }
    }

    pub fn sort_by_tag_value(&mut self, order: &SortAlphabetically) {
        match order {
            SortAlphabetically::AToZ => self.0.sort_by(|a, b| a.tag.value.cmp(&b.tag.value)),
            SortAlphabetically::ZToA => self.0.sort_by(|a, b| b.tag.value.cmp(&a.tag.value)),
        }
    }
}

#[async_trait]
impl FetchAll for Tags {
    /// Get all unique tags (entity & timeline) in the database
    async fn fetch_all(transaction: &mut Transaction<'_, Sqlite>) -> Result<Tags, CrudError> {
        Ok(sqlx::query!(
            r#"
            SELECT
                name AS "name: TagName",
                value AS "value: TagValue"
            FROM entity_tags
            UNION
            SELECT
                name AS "name: TagName",
                value AS "value: TagValue"
            FROM timeline_tags
        "#
        )
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .map(|row| Tag::from(row.name, row.value))
        .collect())
    }
}

/// Fetch all unique entity tags in the database
pub async fn fetch_all_entity_tag_counts(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<TagCounts, CrudError> {
    Ok(sqlx::query!(
        r#"
            SELECT
                name AS "name: TagName",
                value AS "value: TagValue",
                COUNT(*) AS count
            FROM entity_tags
            GROUP BY name, value
        "#
    )
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .map(|row| TagCount {
        tag: Tag::from(row.name, row.value),
        count: row.count,
    })
    .collect())
}

/// Fetch all unique timeline tags in the database
pub async fn fetch_all_timeline_tag_counts(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<TagCounts, CrudError> {
    Ok(sqlx::query!(
        r#"
            SELECT
                name AS "name: TagName",
                value AS "value: TagValue",
                COUNT(*) AS count
            FROM timeline_tags
            GROUP BY name, value
        "#
    )
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .map(|row| TagCount {
        tag: Tag::from(row.name, row.value),
        count: row.count,
    })
    .collect())
}

/// Update all entity tags that match (enables batch editing)
pub async fn update_all_matching_entity_tags(
    transaction: &mut Transaction<'_, Sqlite>,
    old_tag: Tag,
    new_tag: Tag,
) -> Result<RowsAffected, CrudError> {
    Ok(sqlx::query!(
        r#"
            UPDATE entity_tags
            SET
                name = ?,
                value = ?
            WHERE 
                    (name IS ? OR name = ?)
                AND
                    value = ?;
        "#,
        new_tag.name,
        new_tag.value,
        old_tag.name,
        old_tag.name,
        old_tag.value,
    )
    .execute(&mut **transaction)
    .await?
    .rows_affected())
}

// TODO: return RowsAffected?
/// Delete tag from database
pub async fn delete_all_matching_tags(
    transaction: &mut Transaction<'_, Sqlite>,
    tag: Tag,
) -> Result<(), CrudError> {
    // Delete entity tags
    sqlx::query!(
        r#"
            DELETE FROM entity_tags
            WHERE 
                    (name IS ? OR name = ?)
                AND
                    value = ?;
        "#,
        tag.name,
        tag.name,
        tag.value,
    )
    .execute(&mut **transaction)
    .await?;

    // Delete timeline tags
    sqlx::query!(
        r#"
            DELETE FROM timeline_tags
            WHERE 
                    (name IS ? OR name = ?)
                AND
                    value = ?;
        "#,
        tag.name,
        tag.name,
        tag.value,
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
