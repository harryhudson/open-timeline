// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All CRUD functionality for individual timelines ([`TimelineEdit`]s)
//!

use crate::{
    Create, CrudError, DeleteById, DeleteByName, FetchById, FetchByName, IsATimelineType, Update,
    entity_name_from_id, fetch_timeline_bool_expr_string_by_timeline_id,
    fetch_timeline_direct_member_entity_ids_by_timeline_id,
    fetch_timeline_direct_subtimeline_ids_by_timeline_id, fetch_timeline_tags,
    is_timeline_id_in_db, timeline_id_from_name, timeline_name_from_id,
};
use bool_tag_expr::{BoolTagExpr, Tags};
use open_timeline_core::{
    HasIdAndName, IsReducedCollection, IsReducedType, Name, OpenTimelineId, ReducedEntities,
    ReducedEntity, ReducedTimeline, ReducedTimelines, TimelineEdit,
};
use sqlx::{Sqlite, Transaction};
use std::collections::BTreeSet;

// TODO: ensure:
// - No duplicate entities in a timeline (by ID)
// - No duplicate subtimelines (by ID)
// - Timeline isn't a subtimeline of itself

impl IsATimelineType for TimelineEdit {}

impl Create for TimelineEdit {
    // TODO: do anything with the rows_affected() count/value? Applies to other
    // execute()s too
    /// Create a Timeline
    async fn create(&mut self, transaction: &mut Transaction<'_, Sqlite>) -> Result<(), CrudError> {
        // Note: don't throw away an ID if it's set.  If the ID should be thrown
        // away (e.g. when using the PUT /timeline API endpoint) it should be
        // done before calling this function
        // - TODO: should there be checks in case it exists?
        if self.id().is_none() {
            self.set_id(OpenTimelineId::new());
        }

        // Save timeline name
        insert_timeline_id_and_name_and_bool_expr(
            transaction,
            &self.id().unwrap(),
            &self.name(),
            &self.bool_expr(),
        )
        .await?;

        // Save direct entities
        if let Some(entities) = self.entities() {
            let entity_ids: BTreeSet<OpenTimelineId> = entities.ids();
            insert_timeline_direct_entities(transaction, &self.id().unwrap(), entity_ids).await?;
        }

        // Save subtimelines
        if let Some(subtimelines) = self.subtimelines() {
            let subtimeline_ids: BTreeSet<OpenTimelineId> = subtimelines.ids();
            insert_timeline_subtimelines(transaction, &self.id().unwrap(), subtimeline_ids).await?;
        }

        // Save tags
        if let Some(tags) = self.tags() {
            insert_timeline_tags(transaction, &self.id().unwrap(), tags).await?;
        }

        Ok(())
    }
}

impl FetchByName for TimelineEdit {
    async fn fetch_by_name(
        transaction: &mut Transaction<'_, Sqlite>,
        name: &Name,
    ) -> Result<TimelineEdit, CrudError> {
        let id = timeline_id_from_name(transaction, name).await?;
        TimelineEdit::fetch_by_id(transaction, &id).await
    }
}

impl FetchById for TimelineEdit {
    async fn fetch_by_id(
        transaction: &mut Transaction<'_, Sqlite>,
        id: &OpenTimelineId,
    ) -> Result<TimelineEdit, CrudError> {
        if !is_timeline_id_in_db(transaction, id).await? {
            Err(CrudError::IdNotInDb)?
        }

        // TODO: can fetch name and bool expr at same time
        // Name
        let timeline_name = timeline_name_from_id(transaction, id).await?;

        // Bool exprs
        let timeline_bool_expr = fetch_timeline_bool_expr_string_by_timeline_id(transaction, id)
            .await?
            .map(|expr| BoolTagExpr::from(expr).unwrap());

        // Entities
        let timeline_entities =
            match fetch_timeline_direct_member_entity_ids_by_timeline_id(transaction, id).await {
                Err(_) => Err(CrudError::FetchingTimelineDirectMemberEntities)?,
                Ok(None) => None,
                Ok(Some(entity_ids)) => {
                    let mut entities = ReducedEntities::new();
                    for entity_id in entity_ids {
                        let name = entity_name_from_id(transaction, &entity_id).await?;
                        entities
                            .collection_mut()
                            .insert(ReducedEntity::from_id_and_name(entity_id, name));
                    }
                    (!entities.collection().is_empty()).then_some(entities)
                }
            };

        // Subtimelines
        let timeline_subtimelines =
            match fetch_timeline_direct_subtimeline_ids_by_timeline_id(transaction, id).await {
                Err(_) => Err(CrudError::FetchingTimelineDirectSubtimelineIds)?,
                Ok(None) => None,
                Ok(Some(subtimeline_ids)) => {
                    let mut subtimelines = ReducedTimelines::new();
                    for subtimeline_id in subtimeline_ids {
                        let name = timeline_name_from_id(transaction, &subtimeline_id).await?;
                        subtimelines
                            .collection_mut()
                            .insert(ReducedTimeline::from_id_and_name(subtimeline_id, name));
                    }
                    (!subtimelines.collection().is_empty()).then_some(subtimelines)
                }
            };

        // Tags
        let timeline_tags = match fetch_timeline_tags(transaction, id).await {
            Ok(tags) => tags,
            Err(_) => Err(CrudError::FetchingTimelineTags)?,
        };

        Ok(TimelineEdit::from(
            Some(*id),
            timeline_name,
            timeline_bool_expr,
            timeline_entities,
            timeline_subtimelines,
            timeline_tags,
        )
        .unwrap())
    }
}

impl Update for TimelineEdit {
    /// Update a Timeline
    async fn update(&mut self, transaction: &mut Transaction<'_, Sqlite>) -> Result<(), CrudError> {
        // TODO: should this be an error?
        if self.id().is_none() {
            Err(CrudError::IdNotSetForTimeline(self.name().to_owned()))?;
        }
        let timeline_id = self.id().unwrap();
        let timeline_name = self.name();

        // Name & Bool expr
        {
            let bool_expr = self
                .bool_expr()
                .clone()
                .map(|expr| expr.to_boolean_expression());
            let result = sqlx::query!(
                r#"
                    UPDATE timelines
                    SET
                        name = ?,
                        bool_expression = ?
                    WHERE id = ?
                "#,
                timeline_name,
                bool_expr,
                timeline_id,
            )
            .execute(&mut **transaction)
            .await?;

            // TODO: !=1 or >1?
            if result.rows_affected() != 1 {
                Err(CrudError::UpdatingName)?
            }
        }

        // Entities
        {
            // Delete
            delete_timeline_direct_entities(transaction, &timeline_id).await?;

            // Insert
            if let Some(entities) = self.entities() {
                let entity_ids: BTreeSet<OpenTimelineId> = entities.ids();
                insert_timeline_direct_entities(transaction, &timeline_id, entity_ids).await?;
            }
        }

        // Subtimelines
        {
            // Delete
            delete_subtimelines_for_timeline(transaction, &timeline_id).await?;

            // Insert
            if let Some(subtimelines) = self.subtimelines() {
                let subtimeline_ids: BTreeSet<OpenTimelineId> = subtimelines.ids();
                insert_timeline_subtimelines(transaction, &timeline_id, subtimeline_ids).await?;
            }
        }

        // Tags
        {
            // Delete
            delete_timeline_tags(transaction, &timeline_id).await?;

            // Insert
            if let Some(tags) = self.tags() {
                insert_timeline_tags(transaction, &timeline_id, tags).await?;
            }
        }

        Ok(())
    }
}

impl DeleteByName for TimelineEdit {
    async fn delete_by_name(
        transaction: &mut Transaction<'_, Sqlite>,
        name: &Name,
    ) -> Result<(), CrudError> {
        let id = timeline_id_from_name(transaction, name).await?;
        TimelineEdit::delete_by_id(transaction, &id).await
    }
}

impl DeleteById for TimelineEdit {
    async fn delete_by_id(
        transaction: &mut Transaction<'_, Sqlite>,
        id: &OpenTimelineId,
    ) -> Result<(), CrudError> {
        // TODO: check the ID is in the database?

        delete_timeline_tags(transaction, id).await?;
        delete_timeline_direct_entities(transaction, id).await?;
        delete_all_subtimeline_links_for_timeline(transaction, id).await?;

        // This must come last in order to satisfy FOREIGN KEY constraints
        delete_timeline_id_and_name_and_bool_expr(transaction, id).await?;
        Ok(())
    }
}

/// Insert into the database a timeline's name and ID
async fn insert_timeline_id_and_name_and_bool_expr(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
    timeline_name: &Name,
    bool_expr: &Option<BoolTagExpr>,
) -> Result<(), CrudError> {
    let bool_expr = bool_expr.clone().map(|expr| expr.to_boolean_expression());
    sqlx::query!(
        r#"
            INSERT INTO timelines (id, name, bool_expression)
            VALUES (?, ?, ?)
        "#,
        timeline_id,
        timeline_name,
        bool_expr,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

/// Delete from the database a timeline's ID, name, and bool expr
async fn delete_timeline_id_and_name_and_bool_expr(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<(), CrudError> {
    sqlx::query!(
        r#"
            DELETE FROM timelines
            WHERE id=?
        "#,
        timeline_id
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

// TODO: fetch set of entities in all subtimelines, then find the difference
// between that set, and the set of direct entities we're looking to insert.
// Only insert the difference so that no entities that are in subtimelines are
// also inserted into the root timeline (TODO: do the same for subtimelines)
async fn insert_timeline_direct_entities(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
    entity_ids: BTreeSet<OpenTimelineId>,
) -> Result<(), CrudError> {
    for entity_id in entity_ids {
        sqlx::query!(
            r#"
                INSERT INTO timeline_entities (timeline_id, entity_id)
                VALUES (?, ?)
            "#,
            timeline_id,
            entity_id,
        )
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

/// Delete from the database a timeline's direct entities
async fn delete_timeline_direct_entities(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<(), CrudError> {
    sqlx::query!(
        r#"
            DELETE FROM timeline_entities
            WHERE timeline_id = ?
        "#,
        timeline_id,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

// TODO: fetch set of entities in all subtimelines, then find the difference
// between that set, and the set of direct entities we're looking to insert.
// Only insert the difference so that no entities that are in subtimelines are
// also inserted into the root timeline (TODO: do the same for entities)
/// Insert into the database a timeline's subtimelines
async fn insert_timeline_subtimelines(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
    subtimeline_ids: BTreeSet<OpenTimelineId>,
) -> Result<(), CrudError> {
    for subtimeline_id in subtimeline_ids {
        sqlx::query!(
            r#"
                INSERT INTO subtimelines (timeline_parent_id, timeline_child_id) 
                VALUES (?, ?)
            "#,
            timeline_id,
            subtimeline_id
        )
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

/// Delete remove all traces of a timeline from the subtimelines table
/// (both where it is the parent and the child timeline)
async fn delete_all_subtimeline_links_for_timeline(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<(), CrudError> {
    sqlx::query!(
        r#"
            DELETE FROM subtimelines
            WHERE
                    timeline_parent_id=?
                OR
                    timeline_child_id=?
        "#,
        timeline_id,
        timeline_id
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

/// Delete a timeline's subtimelines
async fn delete_subtimelines_for_timeline(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<(), CrudError> {
    sqlx::query!(
        r#"
            DELETE FROM subtimelines
            WHERE timeline_parent_id=?
        "#,
        timeline_id,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

// TODO: test that when tag name is None it is stored as NULL
/// Insert into the database a timeline's tags
async fn insert_timeline_tags(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
    tags: &Tags,
) -> Result<(), CrudError> {
    for tag in tags {
        sqlx::query!(
            r#"
                INSERT INTO timeline_tags (timeline_id, name, value)
                VALUES (?, ?, ?)
            "#,
            timeline_id,
            tag.name,
            tag.value,
        )
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

/// Delete from the database a timeline's tags
async fn delete_timeline_tags(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
) -> Result<(), CrudError> {
    sqlx::query!(
        r#"
            DELETE FROM timeline_tags
            WHERE timeline_id=?
        "#,
        timeline_id
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

/// Add an entity to a database using their IDs
pub async fn insert_timeline_entity(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
    entity_id: &OpenTimelineId,
) -> Result<(), CrudError> {
    if entity_is_direct_member_of_timeline(transaction, entity_id, timeline_id).await? {
        return Ok(());
    }
    sqlx::query!(
        r#"
            INSERT INTO timeline_entities (timeline_id, entity_id)
            VALUES (?, ?)
        "#,
        timeline_id,
        entity_id,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

/// Remove an entity from a database using their IDs
pub async fn delete_timeline_entity(
    transaction: &mut Transaction<'_, Sqlite>,
    timeline_id: &OpenTimelineId,
    entity_id: &OpenTimelineId,
) -> Result<(), CrudError> {
    if !entity_is_direct_member_of_timeline(transaction, entity_id, timeline_id).await? {
        return Ok(());
    }
    sqlx::query!(
        r#"
            DELETE FROM timeline_entities
            WHERE timeline_id=? AND entity_id=?
        "#,
        timeline_id,
        entity_id,
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

/// Check if a timeline has an entity as a direct member using their IDs
async fn entity_is_direct_member_of_timeline(
    transaction: &mut Transaction<'_, Sqlite>,
    entity_id: &OpenTimelineId,
    timeline_id: &OpenTimelineId,
) -> Result<bool, CrudError> {
    Ok(sqlx::query!(
        r#"
            SELECT COUNT(*) AS count
            FROM timeline_entities
            WHERE timeline_id=? AND entity_id=?
        "#,
        timeline_id,
        entity_id,
    )
    .fetch_one(&mut **transaction)
    .await?
    .count
        > 0)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::DatabaseRowCount;
    use crate::test::*;
    use sqlx::Pool;

    mod create {
        use super::*;

        // Uses restore functionality to seed the database as a basic check
        #[sqlx::test]
        async fn basic(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Seed the database with entities and timelines
            seed_db_return_timelines(&mut transaction).await;

            // Check the row counts
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_ne!(row_counts.entities, 0);
            assert_ne!(row_counts.entity_tags, 0);
            assert_ne!(row_counts.subtimelines, 0);
            assert_ne!(row_counts.timeline_entities, 0);
            assert_ne!(row_counts.timeline_tags, 0);
            assert_ne!(row_counts.timelines, 0);
        }

        // ID can be left unset - a new one must be created
        #[sqlx::test]
        async fn id_not_set(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Put some entities into the database (TODO: needed?)
            seed_db_with_entities(&mut transaction).await;

            // Create first timeline (no subtimelines)
            let mut timeline = valid_timeline_no_subtimelines();
            timeline.clear_id();
            assert!(timeline.create(&mut transaction).await.is_ok());

            // Check row counts
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_ne!(row_counts.timelines, 0);
            assert_ne!(row_counts.timeline_tags, 0);
            assert_ne!(row_counts.timeline_entities, 0);
        }

        // If ID already exists, the creation should fail
        #[sqlx::test]
        async fn id_already_exists(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Put some entities into the database (TODO: needed?)
            seed_db_with_entities(&mut transaction).await;

            // Create first timeline (no subtimelines)
            let mut timeline_1 = valid_timeline_no_subtimelines();
            timeline_1.create(&mut transaction).await.unwrap();

            // Attempt to create second timeline with the same ID (no subtimelines)
            let mut timeline_2 = timeline_1.clone();
            timeline_2.set_name(Name::from("other").unwrap());
            assert!(timeline_2.create(&mut transaction).await.is_err());
        }
    }

    mod fetch {
        use super::*;

        #[sqlx::test]
        async fn from_id_and_name(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Put some entities into the database (TODO: needed?)
            seed_db_with_entities(&mut transaction).await;

            // Create first timeline (no subtimelines)
            let mut timeline = valid_timeline_no_subtimelines();
            timeline.create(&mut transaction).await.unwrap();

            // Fetch using name
            let fetched_from_name = TimelineEdit::fetch_by_name(&mut transaction, timeline.name())
                .await
                .unwrap();
            let fetched_from_id =
                TimelineEdit::fetch_by_id(&mut transaction, &timeline.id().unwrap())
                    .await
                    .unwrap();

            // Check
            assert_eq!(timeline, fetched_from_name);
            assert_eq!(timeline, fetched_from_id);
        }
    }

    mod update {
        use super::*;

        #[sqlx::test]
        async fn all_fields(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Put some entities into the database
            seed_db_with_entities(&mut transaction).await;

            // Get 2 different timelines
            let mut timelines = valid_timelines_no_subtimelines();
            let mut timeline = timelines.pop().unwrap();
            let new_timeline = timelines.pop().unwrap();
            assert_ne!(timeline, new_timeline);

            // Create first timeline (no subtimelines)
            timeline.create(&mut transaction).await.unwrap();

            // Create an updated timeline (no subtimelines)
            let original_id = timeline.id().unwrap();
            timeline = new_timeline;
            timeline.set_id(original_id);
            assert!(timeline.update(&mut transaction).await.is_ok());

            // Check row counts
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.timelines, 1);
        }

        #[sqlx::test]
        async fn id_not_set(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Put some entities into the database (TODO: needed?)
            seed_db_with_entities(&mut transaction).await;

            // Create first timeline (no subtimelines)
            let mut timeline = valid_timeline_no_subtimelines();
            timeline.create(&mut transaction).await.unwrap();

            // Check the timeline was  inserted
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.timelines, 1);

            // Clear the ID and attempt to update
            timeline.clear_id();
            assert!(timeline.update(&mut transaction).await.is_err());
        }

        #[sqlx::test]
        async fn not_in_db(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Attempt to update a timeline that's not in the database
            let mut timeline = valid_timeline_no_subtimelines();
            assert!(timeline.update(&mut transaction).await.is_err());
        }

        #[sqlx::test]
        async fn new_name_matches_an_existing_timeline(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Put some entities into the database (TODO: needed?)
            seed_db_with_entities(&mut transaction).await;

            // Create first timeline (no subtimelines)
            let mut timeline_1 = valid_timeline_no_subtimelines();
            timeline_1.create(&mut transaction).await.unwrap();

            // Create second timeline (no subtimelines)
            let mut timeline_2 = valid_timeline_no_subtimelines();
            timeline_2.set_id(OpenTimelineId::new());
            timeline_2.set_name(Name::from("other").unwrap());
            timeline_2.create(&mut transaction).await.unwrap();

            // Set the name of timeline 2 equal to the name of timeline 1
            timeline_2.set_name(timeline_1.name().clone());
            assert!(timeline_2.update(&mut transaction).await.is_err());
        }
    }

    mod delete {
        use super::*;

        #[sqlx::test]
        async fn from_id_and_name(pool: Pool<Sqlite>) {
            // Get the transaction
            let mut transaction = pool.begin().await.unwrap();

            // Put some entities into the database
            seed_db_with_entities(&mut transaction).await;

            // From ID

            // Create timeline (no subtimelines)
            let mut timeline = valid_timeline_no_subtimelines();
            timeline.create(&mut transaction).await.unwrap();

            // Ensure it's in the database
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.timelines, 1);

            // Delete the timeline & check
            let deleted =
                TimelineEdit::delete_by_id(&mut transaction, &timeline.id().unwrap()).await;
            assert!(deleted.is_ok());

            // Check the timeline row counts are all 0
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.subtimelines, 0);
            assert_eq!(row_counts.timeline_entities, 0);
            assert_eq!(row_counts.timeline_tags, 0);
            assert_eq!(row_counts.timelines, 0);

            // From name

            // Create timeline (no subtimelines)
            let mut timeline = valid_timeline_no_subtimelines();
            timeline.create(&mut transaction).await.unwrap();

            // Ensure it's in the database
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.timelines, 1);

            // Delete the timeline & check
            let deleted = TimelineEdit::delete_by_name(&mut transaction, timeline.name()).await;
            assert!(deleted.is_ok());

            // Check the timeline row counts are all 0
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.subtimelines, 0);
            assert_eq!(row_counts.timeline_entities, 0);
            assert_eq!(row_counts.timeline_tags, 0);
            assert_eq!(row_counts.timelines, 0);
        }

        #[sqlx::test]
        fn not_in_db(pool: Pool<Sqlite>) {
            // Get the transaction
            let mut transaction = pool.begin().await.unwrap();

            // Delete an entity that's not in the database
            let name = Name::from("madeup").unwrap();
            let deleted = TimelineEdit::delete_by_name(&mut transaction, &name).await;
            assert!(deleted.is_err());
        }
    }
}
