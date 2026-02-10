// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All functionality that relates only to [`Entity`]s
//!

use crate::{CrudError, FetchById, FetchByPartialName, Limit};
use open_timeline_core::{Entity, OpenTimelineId, ReducedEntities};
use sqlx::{Sqlite, Transaction};

// TODO: much copied form timeline.rs - can any be a macro/generic?

// TODO: seems silly having both of these functions
/// Fetch some number of random entities that have end years
pub async fn fetch_random_entities_with_an_end_year(
    transaction: &mut Transaction<'_, Sqlite>,
    Limit(limit): Limit,
) -> Result<Vec<Entity>, CrudError> {
    let entity_ids = sqlx::query_scalar!(
        r#"
                SELECT id AS "id: OpenTimelineId"
                FROM entities
                WHERE end_year IS NOT NULL
                ORDER BY RANDOM()
                LIMIT ?
            "#,
        limit
    )
    .fetch_all(&mut **transaction)
    .await?;
    let mut entities = Vec::new();
    for entity_id in entity_ids {
        entities.push(Entity::fetch_by_id(transaction, &entity_id).await?);
    }
    Ok(entities)
}

/// Fetch some number of random entities
pub async fn fetch_random_entities(
    transaction: &mut Transaction<'_, Sqlite>,
    Limit(limit): Limit,
) -> Result<Vec<Entity>, CrudError> {
    let reduced_entities =
        ReducedEntities::fetch_by_partial_name(transaction, Limit(limit), "").await?;
    let mut entities = Vec::new();
    for id in reduced_entities.ids() {
        entities.push(Entity::fetch_by_id(transaction, &id).await?);
    }
    Ok(entities)
}
