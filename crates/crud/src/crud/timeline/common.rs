// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Code common to all timeline types
//!

use crate::{CrudError, IdOrName, string_is_name_or_id};
use bool_tag_expr::{Tag, TagName, TagValue, Tags};
use open_timeline_core::{HasIdAndName, Name, OpenTimelineId};
use sqlx::{Sqlite, Transaction};

/// Implementing types are a timeline type (and must also implement
/// [`HasIdAndName`])
pub trait IsATimelineType: HasIdAndName {}

/// Get a timeline's [`Name`] from it's [`OpenTimelineId`]
pub async fn timeline_name_from_id(
    transaction: &mut Transaction<'_, Sqlite>,
    id: &OpenTimelineId,
) -> Result<Name, CrudError> {
    if !is_timeline_id_in_db(transaction, id).await? {
        Err(CrudError::IdNotInDb)?
    }
    Ok(sqlx::query!(
        r#"
            SELECT name AS "name: Name"
            FROM timelines
            WHERE id=?
        "#,
        id
    )
    .fetch_one(&mut **transaction)
    .await?
    .name)
}

/// Get a timeline's [`OpenTimelineId`] from it's [`Name`]
pub async fn timeline_id_from_name(
    transaction: &mut Transaction<'_, Sqlite>,
    name: &Name,
) -> Result<OpenTimelineId, CrudError> {
    if !is_timeline_name_in_db(transaction, name).await? {
        return Err(CrudError::NameNotInDb);
    }
    Ok(sqlx::query!(
        r#"
            SELECT id AS "id: OpenTimelineId"
            FROM timelines
            WHERE name=?
        "#,
        name
    )
    .fetch_one(&mut **transaction)
    .await?
    .id)
}

/// Check if the [`OpenTimelineId`] is a timeline ID in the database
pub async fn is_timeline_id_in_db(
    transaction: &mut Transaction<'_, Sqlite>,
    possible_timeline_id: &OpenTimelineId,
) -> Result<bool, CrudError> {
    Ok(sqlx::query!(
        r#"
            SELECT COUNT(id) AS count
            FROM timelines
            WHERE id=?
        "#,
        possible_timeline_id
    )
    .fetch_one(&mut **transaction)
    .await?
    .count
        > 0)
}

// TODO: is this duplicate functionality of that in crud::timeline.rs?
/// Find out if the given string is a timeline's [`OpenTimelineId`], [`Name`], or
/// neither
pub async fn timeline_id_or_name(
    transaction: &mut Transaction<'_, Sqlite>,
    id_or_name: String,
) -> Result<Option<IdOrName>, CrudError> {
    match string_is_name_or_id(id_or_name) {
        None => Err(CrudError::NeitherIdNorName),
        Some(IdOrName::Id(id)) => {
            if is_timeline_id_in_db(transaction, &id).await? {
                Ok(Some(IdOrName::Id(id)))
            } else {
                Err(CrudError::IdNotInDb)
            }
        }
        Some(IdOrName::Name(name)) => {
            if is_timeline_name_in_db(transaction, &name).await? {
                Ok(Some(IdOrName::Name(name)))
            } else {
                Err(CrudError::NameNotInDb)
            }
        }
    }
}

/// Check if the [`Name`] is a timeline name in the database
pub async fn is_timeline_name_in_db(
    transaction: &mut Transaction<'_, Sqlite>,
    possible_entity_name: &Name,
) -> Result<bool, CrudError> {
    Ok(sqlx::query!(
        r#"
            SELECT COUNT(name) AS count
            FROM timelines
            WHERE name=?
        "#,
        possible_entity_name
    )
    .fetch_one(&mut **transaction)
    .await?
    .count
        > 0)
}

/// Fetch a timeline's [`Tags`]
pub async fn fetch_timeline_tags(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<Option<Tags>, CrudError> {
    let tags: Tags = sqlx::query!(
        r#"
            SELECT
                name AS "name: TagName",
                value AS "value: TagValue"
            FROM timeline_tags
            WHERE timeline_id=?
        "#,
        timeline_id
    )
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .map(|row| Tag::from(row.name, row.value))
    .collect();

    if tags.is_empty() {
        Ok(None)
    } else {
        Ok(Some(tags))
    }
}

/// Fetch the [`OpenTimelineId`] of timelines that are direct (not indirect)
/// subtimelines of the given timeline
pub async fn fetch_timeline_direct_subtimeline_ids_by_timeline_id(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<Option<Vec<OpenTimelineId>>, CrudError> {
    let ids: Vec<OpenTimelineId> = sqlx::query!(
        r#"
            SELECT timeline_child_id AS "timeline_child_id: OpenTimelineId"
            FROM subtimelines
            WHERE timeline_parent_id=?
        "#,
        timeline_id
    )
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .map(|row| row.timeline_child_id)
    .collect();

    if ids.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ids))
    }
}

/// Fetch the [`OpenTimelineId`]s of the entities that are direct (not indirect)
/// members of the given timeline
pub async fn fetch_timeline_direct_member_entity_ids_by_timeline_id(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<Option<Vec<OpenTimelineId>>, CrudError> {
    let entity_ids: Vec<OpenTimelineId> = sqlx::query!(
        r#"
            SELECT entity_id AS "entity_id: OpenTimelineId"
            FROM timeline_entities
            WHERE timeline_id=?
        "#,
        timeline_id
    )
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .map(|row| row.entity_id)
    .collect();

    Ok((!entity_ids.is_empty()).then_some(entity_ids))
}

// TODO: do we need or want this?
/// Get a timeline's entity boolean expression as a string
pub async fn fetch_timeline_bool_expr_string_by_timeline_id(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<Option<String>, CrudError> {
    Ok(sqlx::query_scalar!(
        r#"
            SELECT bool_expression
            FROM timelines
            WHERE id=?
        "#,
        timeline_id
    )
    .fetch_one(&mut **transaction)
    .await?)
}
