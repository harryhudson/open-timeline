// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! [`TimelineView`] and related functionality.  Get a Timeline for viewing
//! (i.e. only fetches)
//!

use crate::{
    CrudError, FetchById, FetchByName, IsATimelineType,
    fetch_timeline_bool_expr_string_by_timeline_id,
    fetch_timeline_direct_member_entity_ids_by_timeline_id,
    fetch_timeline_direct_subtimeline_ids_by_timeline_id, timeline_id_from_name,
    timeline_name_from_id,
};
use bool_tag_expr::BoolTagExpr;
use open_timeline_core::{Entity, HasIdAndName, Name, OpenTimelineId, TimelineView};
use sqlx::{Sqlite, Transaction};
use std::collections::BTreeSet;

// TODO: ensure no duplicate entities in a timeline (by ID)

impl IsATimelineType for TimelineView {}

impl FetchByName for TimelineView {
    async fn fetch_by_name(
        transaction: &mut Transaction<'_, Sqlite>,
        name: &Name,
    ) -> Result<TimelineView, CrudError> {
        let id = timeline_id_from_name(transaction, name).await?;
        TimelineView::fetch_by_id(transaction, &id).await
    }
}

impl FetchById for TimelineView {
    async fn fetch_by_id(
        transaction: &mut Transaction<'_, Sqlite>,
        id: &OpenTimelineId,
    ) -> Result<TimelineView, CrudError> {
        // Get the timeline name
        let timeline_name = timeline_name_from_id(transaction, id).await?;

        // Get all entities
        let mut timeline_entities: Vec<Entity> = Vec::new();
        match fetch_all_timeline_entity_ids_by_timeline_id(transaction, id).await {
            Ok(Some(entity_ids)) => {
                for entity_id in entity_ids {
                    timeline_entities.push(Entity::fetch_by_id(transaction, &entity_id).await?);
                }
            }
            Ok(None) => (),
            Err(_) => Err(CrudError::FetchingTimelineAllEntityIds)?,
        }

        // Clean up the entities
        let timeline_entities = match timeline_entities.is_empty() {
            true => None,
            false => {
                timeline_entities.sort_by_key(|a| a.id().unwrap());
                Some(timeline_entities)
            }
        };

        Ok(TimelineView::from(*id, timeline_name, timeline_entities))
    }
}

/// Fetch from the database the IDs of all entities in a timeline and all of
/// its subtimelines
async fn fetch_all_timeline_entity_ids_by_timeline_id(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<Option<BTreeSet<OpenTimelineId>>, CrudError> {
    let mut entity_ids = BTreeSet::<OpenTimelineId>::new();
    let mut timeline_ids = BTreeSet::<OpenTimelineId>::new();
    timeline_ids.insert(*timeline_id);

    // Get all subtimeline IDs (merged with own ID)
    if let Some(timeline_id_and_subtimeline_ids) =
        fetch_all_subtimeline_ids_plus_own_id_for_timeline_id(transaction, timeline_id).await?
    {
        timeline_ids.extend(timeline_id_and_subtimeline_ids);
    }

    // Get direct timeline entities for all timelines
    for timeline_id in &timeline_ids {
        if let Some(ids) =
            fetch_timeline_direct_member_entity_ids_by_timeline_id(transaction, timeline_id).await?
        {
            entity_ids.extend(ids);
        }
    }

    // Get bool expr entities for all timelines
    for timeline_id in &timeline_ids {
        if let Some(ids) =
            fetch_all_timelines_bool_exprs_entity_ids(transaction, timeline_id).await?
        {
            entity_ids.extend(ids);
        }
    }

    // Return all the entities
    if !entity_ids.is_empty() {
        Ok(Some(entity_ids))
    } else {
        Ok(None)
    }
}

/// Fetch from the database the IDs of all subtimelines of a given timeline
/// (direct and indirect).  Also includes it's own ID
async fn fetch_all_subtimeline_ids_plus_own_id_for_timeline_id(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<Option<Vec<OpenTimelineId>>, CrudError> {
    // Consider making these HashMaps or BTreeMaps
    let mut timeline_ids_processed = Vec::<OpenTimelineId>::new();
    let mut timeline_id_backlog = Vec::<OpenTimelineId>::new();
    timeline_id_backlog.push(*timeline_id);
    loop {
        let id = timeline_id_backlog.pop();
        if id.is_none() {
            break;
        }
        let id = id.unwrap();
        if timeline_ids_processed.contains(&id) {
            continue;
        }
        let ids = fetch_timeline_direct_subtimeline_ids_by_timeline_id(transaction, &id).await?;
        if let Some(mut ids) = ids {
            timeline_id_backlog.append(&mut ids);
        }
        timeline_ids_processed.push(id);
    }
    if !timeline_ids_processed.is_empty() {
        Ok(Some(timeline_ids_processed))
    } else {
        Ok(None)
    }
}

// TODO: rename?
/// Fetch from the database the IDs of all entities that match any of the
/// timeline's boolean expressions
async fn fetch_all_timelines_bool_exprs_entity_ids(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<Option<Vec<OpenTimelineId>>, CrudError> {
    // Vector of boolean expr strings
    let Some(bool_expr) = fetch_timeline_bool_expr_string_by_timeline_id(transaction, timeline_id)
        .await?
        .map(|expr| BoolTagExpr::from(expr).unwrap())
    else {
        return Ok(None);
    };

    let table_info =
        bool_tag_expr::DbTableInfo::from("entity_tags", "entity_id", "name", "value").unwrap();

    // Vector of boolean expression SQL statements
    let bool_expr_sql: String = bool_expr.to_sql(&table_info);

    // All entity IDs fetched using boolean expressions
    let mut entity_ids = BTreeSet::new();
    let sql = format!(
        r#"
                SELECT DISTINCT entity_id AS "entity_id: OpenTimelineId"
                FROM ({bool_expr_sql})
            "#,
    );
    let new_entity_ids: Vec<OpenTimelineId> = sqlx::query_scalar(&sql)
        .fetch_all(&mut **transaction)
        .await?;
    entity_ids.extend(new_entity_ids);

    Ok((!entity_ids.is_empty()).then_some(entity_ids.into_iter().collect()))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use sqlx::Pool;

    #[sqlx::test]
    async fn fetch_timelines_bool_expr(pool: Pool<Sqlite>) {
        // Setup
        let mut transaction = pool.begin().await.unwrap();

        // Seed the database
        seed_db(&mut transaction).await;

        // Get a timeline with a bool expr
        let timeline = valid_timeline_with_bool_expr();

        // Fetch the bool expr using the timeline's ID
        let bool_expr_fetched = fetch_timeline_bool_expr_string_by_timeline_id(
            &mut transaction,
            &timeline.id().unwrap(),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(
            bool_expr_fetched,
            timeline
                .bool_expr()
                .clone()
                .unwrap()
                .to_boolean_expression()
        );
    }

    // TODO: this could be improved
    #[sqlx::test]
    async fn timelines_entities_using_its_bool_exprs(pool: Pool<Sqlite>) {
        // Setup
        let mut transaction = pool.begin().await.unwrap();

        // Seed the database
        seed_db(&mut transaction).await;

        // Get a timeline with a bool expr
        let timeline = valid_timeline_with_bool_expr();

        // Get the IDs of the entities whose tags match the timeline's bool expr (fetched using its ID)
        let entity_ids =
            fetch_all_timelines_bool_exprs_entity_ids(&mut transaction, &timeline.id().unwrap())
                .await
                .unwrap()
                .unwrap();

        assert_ne!(entity_ids.len(), 0);
    }

    mod fetch {
        use super::*;

        // TODO: check that the fetching by bool expr and subtimelines work
        #[sqlx::test]
        async fn all(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Seed the database
            seed_db(&mut transaction).await;

            // Get a timeline
            let timeline = valid_timelines().pop().unwrap();

            // Fetch the timeline using its name
            let timeline_fetched_by_name =
                TimelineView::fetch_by_name(&mut transaction, &timeline.name())
                    .await
                    .unwrap();

            // Fetch the timeline using its ID
            let timeline_fetched_by_id =
                TimelineView::fetch_by_id(&mut transaction, &timeline.id().unwrap())
                    .await
                    .unwrap();

            // Check
            assert_eq!(timeline_fetched_by_name, timeline_fetched_by_id);
        }
    }
}
