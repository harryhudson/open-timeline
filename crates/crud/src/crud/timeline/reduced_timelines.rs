// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! CRUD trait implementations for [`ReducedTimelines`]
//!

use crate::{
    CrudError, FetchAll, FetchAllWithTag, FetchByBoolTagExpr, FetchById, FetchByPartialName,
    FetchByPartialNameAndBoolTagExpr, Limit,
};
use async_trait::async_trait;
use bool_tag_expr::{BoolTagExpr, Tag};
use open_timeline_core::{
    IsReducedCollection, IsReducedType, Name, OpenTimelineId, ReducedTimeline, ReducedTimelines,
};
use sqlx::{Sqlite, Transaction};

#[async_trait]
impl FetchAll for ReducedTimelines {
    /// Get all timelines.
    async fn fetch_all(
        transaction: &mut Transaction<'_, Sqlite>,
    ) -> Result<ReducedTimelines, CrudError> {
        Ok(sqlx::query!(
            r#"
                SELECT DISTINCT
                    id AS "id: OpenTimelineId",
                    name AS "name: Name"
                FROM timelines
            "#
        )
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .map(|row| ReducedTimeline::from_id_and_name(row.id, row.name))
        .collect())
    }
}

#[async_trait]
impl FetchAllWithTag for ReducedTimelines {
    /// Get all timelines that have the given tag.
    ///
    /// For more complicated tag-related queries, use the boolean expression
    /// functionality.
    async fn fetch_all_with_tag(
        transaction: &mut Transaction<'_, Sqlite>,
        tag: &Tag,
    ) -> Result<ReducedTimelines, CrudError> {
        Ok(sqlx::query!(
            r#"
            SELECT DISTINCT
                timelines.id AS "id: OpenTimelineId",
                timelines.name AS "name: Name"
            FROM timelines
            JOIN timeline_tags ON timelines.id = timeline_tags.timeline_id
            WHERE
            (
                timeline_tags.name = ?
                OR (timeline_tags.name IS NULL AND ? IS NULL)
            )
            AND timeline_tags.value = ?
        "#,
            tag.name,
            tag.name,
            tag.value
        )
        .fetch_all(&mut **transaction)
        .await?
        .into_iter()
        .map(|row| ReducedTimeline::from_id_and_name(row.id, row.name))
        .collect())
    }
}

#[async_trait]
impl FetchByBoolTagExpr for ReducedTimelines {
    // TODO: given we're generating SQL, this probably needs better error
    // checking.  On the other hand, one can't SQL inject because of char
    // restrictions
    /// Fetch all entities that match a [`BoolTagExpr`]
    async fn fetch_by_bool_tag_expr(
        transaction: &mut Transaction<'_, Sqlite>,
        Limit(limit): Limit,
        bool_expr: BoolTagExpr,
    ) -> Result<Self, CrudError> {
        let table_info =
            bool_tag_expr::DbTableInfo::from("timeline_tags", "timeline_id", "name", "value")
                .unwrap();

        let bool_expr_sql = bool_expr.to_sql(&table_info);

        let sql = format!(
            r#"
                SELECT DISTINCT timeline_id AS "timeline_id: OpenTimelineId"
                FROM ({bool_expr_sql})
                LIMIT ?
            "#,
        );

        // TODO: does this work? Needs testing
        let timeline_ids: Vec<OpenTimelineId> = sqlx::query_scalar(&sql)
            .bind(limit)
            .fetch_all(&mut **transaction)
            .await?;

        let mut reduced_timelines = ReducedTimelines::new();
        for timeline_id in timeline_ids {
            let reduced_entity = ReducedTimeline::fetch_by_id(transaction, &timeline_id).await?;
            reduced_timelines.collection_mut().insert(reduced_entity);
        }

        Ok(reduced_timelines)
    }
}

#[async_trait]
impl FetchByPartialName for ReducedTimelines {
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
                FROM timelines
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
        .map(|row| ReducedTimeline::from_id_and_name(row.id, row.name))
        .collect())
    }
}

// TODO: do properly with JOIN(s)
#[async_trait]
impl FetchByPartialNameAndBoolTagExpr for ReducedTimelines {
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
