// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All CRUD functionality for individual [`Entity`]s
//!

use crate::crud::common::*;
use crate::crud::common::{Create, Update};
use bool_tag_expr::{Tag, TagName, TagValue, Tags};
use open_timeline_core::{Date, Entity, HasIdAndName, Name, OpenTimelineId};
use sqlx::{Sqlite, Transaction};

impl Create for Entity {
    /// Create an [`Entity`] in the database
    async fn create(&mut self, transaction: &mut Transaction<'_, Sqlite>) -> Result<(), CrudError> {
        if self.id().is_none() {
            self.set_id(OpenTimelineId::new());
        }

        // ID, Name, and Dates
        {
            let entity_id = self.id().unwrap();
            let entity_name = self.name();
            let start_year = self.start_year();
            let start_month = self.start_month();
            let start_day = self.start_day();
            let end_year = self.end_year();
            let end_month = self.end_month();
            let end_day = self.end_day();

            sqlx::query!(
                r#"
                INSERT INTO entities
                (
                    id,
                    name,
                    start_year,
                    start_month,
                    start_day,
                    end_year,
                    end_month,
                    end_day
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
                entity_id,
                entity_name,
                start_year,
                start_month,
                start_day,
                end_year,
                end_month,
                end_day
            )
            .execute(&mut **transaction)
            .await
            .map_err(|error| {
                if let Some(db_err) = error.as_database_error() {
                    if db_err.is_unique_violation() {
                        return CrudError::EntityNameAlreadyInUse(entity_name.clone());
                    }
                }
                CrudError::into(error.into())
            })?;
        }

        // Tags
        if let Some(tags) = &self.tags() {
            insert_entity_tags(transaction, &self.id().unwrap(), tags).await?;
        }

        Ok(())
    }
}

impl FetchByName for Entity {
    async fn fetch_by_name(
        transaction: &mut Transaction<'_, Sqlite>,
        name: &Name,
    ) -> Result<Entity, CrudError> {
        let id = entity_id_from_name(transaction, name).await?;
        Entity::fetch_by_id(transaction, &id).await
    }
}

impl FetchById for Entity {
    // TODO: might be able to use this in generics if we remove `async` and instead
    // return `Pin<Box<dyn Future<Output = Result<Entity, CrudError>> + Send + 'static>>;`
    // But that might be a pain (macro for now - see gui/main.rs)
    async fn fetch_by_id(
        transaction: &mut Transaction<'_, Sqlite>,
        id: &OpenTimelineId,
    ) -> Result<Self, CrudError> {
        match is_entity_id_in_db(transaction, id).await {
            Ok(true) => (),
            Ok(false) => return Err(CrudError::IdNotInDb),
            Err(_) => return Err(CrudError::DbError),
        }

        // NOTE: the "id: OpenTimelineId" is essential
        // Name & Dates
        let (entity_name, entity_start, entity_end) = {
            let record = sqlx::query!(
                r#"
                SELECT
                    id AS "id: OpenTimelineId",
                    name AS "name: Name",
                    start_year,
                    start_month,
                    start_day,
                    end_year,
                    end_month,
                    end_day
                FROM entities
                WHERE id=?
            "#,
                id
            )
            .fetch_one(&mut **transaction)
            .await?;

            // Name
            let name = record.name;

            // Start date
            let start = Date::from(record.start_day, record.start_month, record.start_year)
                .map_err(|_| CrudError::Date)?;

            // End date
            let end = if let Some(end_year) = record.end_year {
                Some(
                    Date::from(record.end_day, record.end_month, end_year)
                        .map_err(|_| CrudError::Date)?,
                )
            } else {
                None
            };
            (name, start, end)
        };

        // Tags
        let entity_tags = {
            let tags: Tags = sqlx::query!(
                r#"
                    SELECT
                        name AS "name: TagName",
                        value AS "value: TagValue"
                    FROM entity_tags
                    WHERE entity_id=?
                "#,
                id
            )
            .fetch_all(&mut **transaction)
            .await?
            .into_iter()
            .map(|row| Tag::from(row.name, row.value))
            .collect();

            (!tags.is_empty()).then_some(tags)
        };

        // Return entity
        Entity::from(
            Some(*id),
            entity_name,
            entity_start,
            entity_end,
            entity_tags,
        )
        .map_err(|_| CrudError::Name)
    }
}

impl Update for Entity {
    /// Update an Entity
    async fn update(&mut self, transaction: &mut Transaction<'_, Sqlite>) -> Result<(), CrudError> {
        if self.id().is_none() {
            return Err(CrudError::IdNotSetForEntity(self.name().to_owned()));
        }
        let entity_id = self.id().unwrap();
        let entity_name = self.name();

        // Name
        {
            // TODO: check if update, or if nothing to update (ie failed)
            match sqlx::query!(
                r#"
                    UPDATE entities
                    SET name = ?
                    WHERE id = ?
                "#,
                entity_name,
                entity_id,
            )
            .execute(&mut **transaction)
            .await
            {
                Ok(result) => {
                    if result.rows_affected() != 1 {
                        Err(CrudError::UpdatingName)?
                    }
                }
                Err(_) => {
                    // TODO: likely the name already exists
                    Err(CrudError::UpdatingName)?
                }
            };
        }

        // Dates
        {
            let start_year = self.start_year();
            let start_month = self.start_month();
            let start_day = self.start_day();
            let end_year = self.end_year();
            let end_month = self.end_month();
            let end_day = self.end_day();
            sqlx::query!(
                r#"UPDATE entities
                SET
                    start_year = ?,
                    start_month = ?,
                    start_day = ?,
                    end_year = ?,
                    end_month = ?,
                    end_day = ?
                WHERE id = ?
            "#,
                start_year,
                start_month,
                start_day,
                end_year,
                end_month,
                end_day,
                entity_id,
            )
            .execute(&mut **transaction)
            .await?;
        }

        // Tags
        {
            delete_entity_tags(transaction, &self.id().unwrap()).await?;
            if let Some(tags) = &self.tags() {
                insert_entity_tags(transaction, &self.id().unwrap(), tags).await?;
            }
        }

        Ok(())
    }
}

impl DeleteByName for Entity {
    async fn delete_by_name(
        transaction: &mut Transaction<'_, Sqlite>,
        name: &Name,
    ) -> Result<(), CrudError> {
        let id = entity_id_from_name(transaction, name).await?;
        Entity::delete_by_id(transaction, &id).await
    }
}

// TODO: should this fail if it's not in the database?
impl DeleteById for Entity {
    async fn delete_by_id(
        transaction: &mut Transaction<'_, Sqlite>,
        id: &OpenTimelineId,
    ) -> Result<(), CrudError> {
        // From timelines
        delete_entity_from_timelines(transaction, id).await?;

        // Tags
        delete_entity_tags(transaction, id).await?;

        // ID, Name and Dates
        sqlx::query!(
            r#"
                DELETE FROM entities
                WHERE id=?
            "#,
            id
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}

/// Insert and entity's tags into the database
async fn insert_entity_tags(
    transaction: &mut Transaction<'_, Sqlite>,
    entity_id: &OpenTimelineId,
    tags: &Tags,
) -> Result<(), CrudError> {
    for tag in tags {
        sqlx::query!(
            r#"
                INSERT INTO entity_tags (entity_id, name, value)
                VALUES (?, ?, ?)
            "#,
            entity_id,
            tag.name,
            tag.value
        )
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

/// Delete an entity's tags from the database
async fn delete_entity_tags(
    transaction: &mut Transaction<'_, Sqlite>,
    entity_id: &OpenTimelineId,
) -> Result<(), CrudError> {
    sqlx::query!(
        r#"
            DELETE FROM entity_tags
            WHERE entity_id=?
        "#,
        entity_id
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

/// Delete entity from timelines
async fn delete_entity_from_timelines(
    transaction: &mut Transaction<'_, Sqlite>,
    entity_id: &OpenTimelineId,
) -> Result<(), CrudError> {
    sqlx::query!(
        r#"
            DELETE FROM timeline_entities
            WHERE entity_id=?
        "#,
        entity_id
    )
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

/// Check if the [`OpenTimelineId`] is an entity ID in the database
pub async fn is_entity_id_in_db(
    transaction: &mut Transaction<'_, Sqlite>,
    possible_entity_id: &OpenTimelineId,
) -> Result<bool, CrudError> {
    Ok(sqlx::query!(
        r#"
            SELECT COUNT(id) AS count
            FROM entities
            WHERE id=?
        "#,
        possible_entity_id
    )
    .fetch_one(&mut **transaction)
    .await?
    .count
        > 0)
}

/// Check if the [`Name`] is an entity name in the database
pub async fn is_entity_name_in_db(
    transaction: &mut Transaction<'_, Sqlite>,
    string: &Name,
) -> Result<bool, CrudError> {
    Ok(sqlx::query!(
        r#"
            SELECT COUNT(name) AS count
            FROM entities
            WHERE name=?
        "#,
        string
    )
    .fetch_one(&mut **transaction)
    .await?
    .count
        > 0)
}

// TODO: Almost a perfect copy of timeline_id_or_name (merge?)
/// If the string could be an ID or Name, check the database
pub async fn entity_id_or_name(
    transaction: &mut Transaction<'_, Sqlite>,
    id_or_name: String,
) -> Result<Option<IdOrName>, CrudError> {
    match string_is_name_or_id(id_or_name) {
        None => Err(CrudError::NeitherIdNorName),
        Some(IdOrName::Id(id)) => {
            if is_entity_id_in_db(transaction, &id).await? {
                Ok(Some(IdOrName::Id(id)))
            } else {
                Err(CrudError::IdNotInDb)
            }
        }
        Some(IdOrName::Name(name)) => {
            if is_entity_name_in_db(transaction, &name).await? {
                Ok(Some(IdOrName::Name(name)))
            } else {
                Err(CrudError::NameNotInDb)
            }
        }
    }
}

/// Fetch the entity's name from the database using its ID
pub async fn entity_name_from_id(
    transaction: &mut Transaction<'_, Sqlite>,
    id: &OpenTimelineId,
) -> Result<Name, CrudError> {
    // TODO: is this needed? It's not in the timeline equivalent
    if !is_entity_id_in_db(transaction, id).await? {
        return Err(CrudError::IdNotInDb);
    }
    Ok(sqlx::query!(
        r#"
            SELECT name AS "name: Name"
            FROM entities
            WHERE id=?
        "#,
        id
    )
    .fetch_one(&mut **transaction)
    .await?
    .name)
}

// TODO: should this be a method of the HasNameAndId trait (along with other functions?)
/// Fetch the entity's ID from the database using its name
pub async fn entity_id_from_name(
    transaction: &mut Transaction<'_, Sqlite>,
    name: &Name,
) -> Result<OpenTimelineId, CrudError> {
    if !is_entity_name_in_db(transaction, name).await? {
        return Err(CrudError::NameNotInDb);
    }
    Ok(sqlx::query!(
        r#"
            SELECT id AS "id: OpenTimelineId"
            FROM entities
            WHERE name=?
        "#,
        name
    )
    .fetch_one(&mut **transaction)
    .await?
    .id)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::DatabaseRowCount;
    use crate::test::*;
    use open_timeline_core::Entity;
    use sqlx::Pool;

    mod create {
        use super::*;

        // Basic entity creation test where all fields are set
        #[sqlx::test]
        fn all_entity_fields_set(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            let entities = valid_entities();
            for mut entity in entities {
                assert!(entity.create(&mut transaction).await.is_ok());
            }

            // Check row counts
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 3);
        }

        // If the ID is not set it should be saved and an ID created for it
        #[sqlx::test]
        async fn id_not_set(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            let mut entity = valid_entity();
            entity.clear_id();
            assert!(entity.create(&mut transaction).await.is_ok());
            assert!(entity.id().is_some());

            // Check the database row count
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 1);
        }

        // If ID already exists, the creation should fail
        #[sqlx::test]
        async fn id_already_exists(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Create the entity
            let mut entity = valid_entity();
            entity.clear_id();
            assert!(entity.create(&mut transaction).await.is_ok());

            // Assert the row count is correct
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 1);

            // Check that an with an ID already in the database cannot be created
            let mut other_entity = entity.clone();
            other_entity.set_name(Name::from("Other").unwrap());
            assert!(other_entity.create(&mut transaction).await.is_err());

            // Assert the row count is unchanged
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 1);
        }

        // If name already exists, the creation should fail
        //
        // Try create()ing the same Timeline again - UNIQUE key constrait shold
        // fail on the "name"
        //
        // TODO: can parse the database error string to get that it was the "name"
        // column, but need to test it so that I can catch if the msg format
        // changes
        #[sqlx::test]
        async fn name_already_exists(pool: Pool<Sqlite>) {
            // Setup
            let mut transaction = pool.begin().await.unwrap();

            // Create the entity
            let mut entity = valid_entity();
            entity.clear_id();
            assert!(entity.create(&mut transaction).await.is_ok());

            // Assert the row count is correct
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 1);

            // Check that an with an ID already in the database cannot be created
            let mut other_entity = entity.clone();
            other_entity.set_id(OpenTimelineId::new());
            assert!(other_entity.create(&mut transaction).await.is_err());

            // Assert the row count is unchanged
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 1);
        }
    }

    mod fetch {
        use super::*;

        #[sqlx::test]
        fn by_name_and_by_id(pool: Pool<Sqlite>) {
            // Get the transaction
            let mut transaction = pool.begin().await.unwrap();

            // Inset an entity into the database
            let mut entity = valid_entity();
            entity.create(&mut transaction).await.unwrap();

            // Fetch using name
            let fetched_by_name = Entity::fetch_by_name(&mut transaction, entity.name())
                .await
                .unwrap();

            // Fetch using ID
            let fetched_by_id = Entity::fetch_by_id(&mut transaction, &entity.id().unwrap())
                .await
                .unwrap();

            // Check
            assert_eq!(fetched_by_name, entity);
            assert_eq!(fetched_by_id, entity);
        }
    }

    mod update {
        use super::*;

        // Update all fields of an entity that can be updated (not ID)
        #[sqlx::test]
        fn all_fields(pool: Pool<Sqlite>) {
            // Get the transaction
            let mut transaction = pool.begin().await.unwrap();

            let mut entities = valid_entities();

            // Insert original into the database
            let mut original_entity = entities.pop().unwrap();
            original_entity.create(&mut transaction).await.unwrap();

            // Create a new entity with the same ID and call `update()`
            let mut updated_entity = entities.pop().unwrap();
            assert_ne!(original_entity, updated_entity);
            updated_entity.set_id(original_entity.id().unwrap());
            assert!(updated_entity.update(&mut transaction).await.is_ok());

            // Check update
            assert_ne!(original_entity, updated_entity);

            // Assert the row count
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 1);
        }

        // ID is not set
        #[sqlx::test]
        fn id_not_set(pool: Pool<Sqlite>) {
            // Get the transaction
            let mut transaction = pool.begin().await.unwrap();

            // Insert into the database
            let mut entity = valid_entity();
            entity.create(&mut transaction).await.unwrap();

            // Clear the ID
            entity.clear_id();

            // Check update fails
            assert!(entity.update(&mut transaction).await.is_err());

            // Assert the row count
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 1);
        }

        // Can't update an entity that isn't in the database
        #[sqlx::test]
        fn not_in_db(pool: Pool<Sqlite>) {
            // Get the transaction
            let mut transaction = pool.begin().await.unwrap();

            // Attempt to update an entity not in the database
            let mut entity = valid_entity();
            assert!(entity.update(&mut transaction).await.is_err());

            // Assert the row count
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 0);
        }

        // The name matches an entity which has a different ID
        #[sqlx::test]
        async fn new_name_matches_an_existing_entity(pool: Pool<Sqlite>) {
            // Get the transaction
            let mut transaction = pool.begin().await.unwrap();

            // Get 2 entities
            let mut entities = valid_entities();
            let mut entity_1 = entities.pop().unwrap();
            let mut entity_2 = entities.pop().unwrap();
            assert_ne!(entity_1, entity_2);

            // Insert entity them into the database
            entity_1.create(&mut transaction).await.unwrap();
            entity_2.create(&mut transaction).await.unwrap();

            // Set entity 2's name equal to entity 1 and check it can't be updated
            entity_2.set_name(entity_1.name().clone());
            assert!(entity_2.update(&mut transaction).await.is_err());

            // Assert the row count
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 2);
        }
    }

    mod delete {
        use super::*;

        #[sqlx::test]
        fn by_name_and_by_id(pool: Pool<Sqlite>) {
            // Get the transaction
            let mut transaction = pool.begin().await.unwrap();

            // Create 2 entities
            let mut entities = valid_entities();
            let mut entity_1 = entities.pop().unwrap();
            let mut entity_2 = entities.pop().unwrap();
            entity_1.create(&mut transaction).await.unwrap();
            entity_2.create(&mut transaction).await.unwrap();

            // Assert the row count
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 2);

            // Delete entity 1 using its ID
            let deleted_1 = Entity::delete_by_id(&mut transaction, &entity_1.id().unwrap()).await;
            // Delete entity 2 using its name
            let deleted_2 = Entity::delete_by_name(&mut transaction, entity_2.name()).await;

            // Assert delettions
            assert!(deleted_1.is_ok());
            assert!(deleted_2.is_ok());

            // Assert the row count
            let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
            assert_eq!(row_counts.entities, 0);
        }

        #[sqlx::test]
        fn not_in_db(pool: Pool<Sqlite>) {
            // Get the transaction
            let mut transaction = pool.begin().await.unwrap();

            // Attempt to delete an entity that's not in the database
            let deleted = Entity::delete_by_id(&mut transaction, &OpenTimelineId::new()).await;

            // Assert the deletion "passed"
            assert!(deleted.is_ok());
        }
    }
}
