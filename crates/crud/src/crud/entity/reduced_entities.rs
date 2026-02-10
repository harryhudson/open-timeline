// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! CRUD trait implementations for [`ReducedEntities`]
//!

use crate::{
    CrudError, FetchAll, FetchAllWithTag, FetchByBoolTagExpr, FetchById, FetchByPartialName,
    FetchByPartialNameAndBoolTagExpr, Limit,
};
use async_trait::async_trait;
use bool_tag_expr::{BoolTagExpr, Tag};
use open_timeline_core::{
    IsReducedCollection, IsReducedType, Name, OpenTimelineId, ReducedEntities, ReducedEntity,
};
use sqlx::{Sqlite, Transaction};

#[async_trait]
impl FetchAll for ReducedEntities {
    /// Get all entities.
    async fn fetch_all(
        transaction: &mut Transaction<'_, Sqlite>,
    ) -> Result<ReducedEntities, CrudError> {
        Ok(sqlx::query!(
            r#"
                SELECT DISTINCT
                    id AS "id: OpenTimelineId",
                    name AS "name: Name"
                FROM entities
            "#
        )
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .map(|row| ReducedEntity::from_id_and_name(row.id, row.name))
        .collect())
    }
}

#[async_trait]
impl FetchAllWithTag for ReducedEntities {
    /// Get all entities that have the given tag.
    ///
    /// For more complicated tag-related queries, use the boolean expression
    /// functionality.
    async fn fetch_all_with_tag(
        transaction: &mut Transaction<'_, Sqlite>,
        tag: &Tag,
    ) -> Result<ReducedEntities, CrudError> {
        Ok(sqlx::query!(
            r#"
            SELECT DISTINCT
                entities.id AS "id: OpenTimelineId",
                entities.name AS "name: Name"
            FROM entities
            JOIN entity_tags ON entities.id = entity_tags.entity_id
            WHERE
            (
                entity_tags.name = ?
                OR (entity_tags.name IS NULL AND ? IS NULL)
            )
            AND entity_tags.value = ?
        "#,
            tag.name,
            tag.name,
            tag.value
        )
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .map(|row| ReducedEntity::from_id_and_name(row.id, row.name))
        .collect())
    }
}

#[async_trait]
impl FetchByBoolTagExpr for ReducedEntities {
    // TODO: given we're generating SQL, this probably needs better error checking
    /// Fetch all entities that match a [`BoolTagExpr`]
    async fn fetch_by_bool_tag_expr(
        transaction: &mut Transaction<'_, Sqlite>,
        Limit(limit): Limit,
        bool_expr: BoolTagExpr,
    ) -> Result<Self, CrudError> {
        let table_info =
            bool_tag_expr::DbTableInfo::from("entity_tags", "entity_id", "name", "value").unwrap();

        let bool_expr_sql = bool_expr.to_sql(&table_info);

        let sql = format!(
            r#"
                SELECT DISTINCT entity_id  AS "entity_id: OpenTimelineId"
                FROM ({bool_expr_sql})
                LIMIT ?
            "#
        );

        // TODO: does this work? Needs testing
        let entity_ids: Vec<OpenTimelineId> = sqlx::query_scalar(&sql)
            .bind(limit)
            .fetch_all(&mut **transaction)
            .await?;

        let mut reduced_entities = ReducedEntities::new();
        for entity_id in entity_ids {
            let reduced_entity = ReducedEntity::fetch_by_id(transaction, &entity_id).await?;
            reduced_entities.collection_mut().insert(reduced_entity);
        }

        Ok(reduced_entities)
    }
}

#[async_trait]
impl FetchByPartialName for ReducedEntities {
    async fn fetch_by_partial_name(
        transaction: &mut Transaction<'_, Sqlite>,
        Limit(limit): Limit,
        partial_name: &str,
    ) -> Result<Self, CrudError> {
        let partial_name = partial_name.to_string();
        Ok(sqlx::query!(
            r#"
                SELECT
                    id AS "id: OpenTimelineId",
                    name AS "name: Name"
                FROM entities
                WHERE name LIKE CONCAT('%', ?, '%')
                ORDER BY RANDOM()
                LIMIT ?
            "#,
            partial_name,
            limit
        )
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .map(|row| ReducedEntity::from_id_and_name(row.id, row.name))
        .collect())
    }
}

// TODO: do properly with JOIN(s)
#[async_trait]
impl FetchByPartialNameAndBoolTagExpr for ReducedEntities {
    async fn fetch_by_partial_name_and_bool_tag_expr(
        transaction: &mut Transaction<'_, Sqlite>,
        Limit(limit): Limit,
        partial_name: &str,
        bool_tag_expr: BoolTagExpr,
    ) -> Result<Self, CrudError> {
        let from_expr =
            Self::fetch_by_bool_tag_expr(transaction, Limit(u32::MAX), bool_tag_expr).await?;
        let from_name =
            Self::fetch_by_partial_name(transaction, Limit(u32::MAX), partial_name).await?;
        let both = from_expr
            .collection()
            .intersection(from_name.collection())
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(both)
    }
}
