// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Proper searching
//!

use crate::{CrudError, FetchById, Limit};
use bool_tag_expr::BoolTagExpr;
use open_timeline_core::{
    Date, Entity, IsReducedCollection, Name, OpenTimelineId, ReducedEntities, ReducedEntity,
};
use serde::Deserialize;
use serde::Serialize;
use sqlx::{Sqlite, Transaction};
use std::collections::BTreeSet;
use std::fmt::Display;

/// Possible comparisons/orderings to be used for filtering by [`Date`]
#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SearchDateOrdering {
    LessThan,
    GreaterThan,
    EqualTo,
    NotEqualTo,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
}

impl Display for SearchDateOrdering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::LessThan => "<",
            Self::GreaterThan => ">",
            Self::EqualTo => "==",
            Self::NotEqualTo => "!=",
            Self::LessThanOrEqualTo => "<=",
            Self::GreaterThanOrEqualTo => ">=",
        };
        write!(f, "{str}")
    }
}

/// Ordering options when searching
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum OrderBy {
    Random,
    Name,
    Id,
}

/// Represents a date filter (used to filter by start and end)
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct DateFilter {
    /// The `Date` to filter by
    date: Date,

    /// How to filter by the date (before, at the time of, etc)
    ordering: SearchDateOrdering,
}

/// Full search & filtering for [`Entity`]s
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct EntitySearch {
    /// Filter by name
    partial_name: Option<Name>,

    /// Filter by start date
    start: Option<DateFilter>,

    /// Filter by end date
    end: Option<DateFilter>,

    /// Filter by bool expr
    bool_expr: Option<BoolTagExpr>,

    /// How to order the results
    order_results_by: Option<OrderBy>,

    /// Limit the maximum number of results
    limit: Option<Limit>,
}

impl EntitySearch {
    pub fn from(
        partial_name: Option<Name>,
        start: Option<DateFilter>,
        end: Option<DateFilter>,
        bool_expr: Option<BoolTagExpr>,
        order_by: Option<OrderBy>,
        limit: Option<Limit>,
    ) -> Self {
        Self {
            partial_name,
            start,
            end,
            bool_expr,
            order_results_by: order_by,
            limit,
        }
    }

    // TODO: I think it's be nicer to have Search::for_reduced_entities()
    // But also impl for ReducedEntities et al so can do generic T::search()
    pub async fn fetch_entities(
        transaction: &mut Transaction<'_, Sqlite>,
        search: &EntitySearch,
    ) -> Result<Vec<Entity>, CrudError> {
        let entity_ids = fetch_entity_ids_using_search(transaction, search).await?;

        let mut entities = BTreeSet::new();
        for id in entity_ids {
            entities.insert(Entity::fetch_by_id(transaction, &id).await?);
        }
        Ok(entities.into_iter().collect())
    }

    pub async fn fetch_reduced_entities(
        transaction: &mut Transaction<'_, Sqlite>,
        search: &EntitySearch,
    ) -> Result<ReducedEntities, CrudError> {
        let entity_ids = fetch_entity_ids_using_search(transaction, search).await?;

        let mut reduced_entities = ReducedEntities::new();
        for id in entity_ids {
            reduced_entities
                .collection_mut()
                .insert(ReducedEntity::fetch_by_id(transaction, &id).await?);
        }
        Ok(reduced_entities)
    }
}

// TODO: Could fetch ReducedEntities (id, name), then get entities if wanted
async fn fetch_entity_ids_using_search(
    transaction: &mut Transaction<'_, Sqlite>,
    search: &EntitySearch,
) -> Result<Vec<OpenTimelineId>, CrudError> {
    // WHERE clauses
    let where_clauses = {
        let mut where_clauses = Vec::new();

        // Partial name
        if search.partial_name.is_some() {
            where_clauses.push(String::from("name LIKE CONCAT('%', ?, '%')"));
        }

        // Start
        if let Some(start) = search.start.as_ref() {
            where_clauses.push(create_date_cmp_sql(StartOrEnd::Start, start));
        }

        // End
        if let Some(end) = search.end.as_ref() {
            where_clauses.push(create_date_cmp_sql(StartOrEnd::End, end));
        }

        if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        }
    };

    // TODO: this get ruined if there's a bool expr (i.e. if there's a bool expr
    // sort with Rust not SQL), unless we JOIN!!!
    // ORDER BY
    let order_by_clause = if let Some(ordering) = search.order_results_by.as_ref() {
        match ordering {
            OrderBy::Id => String::from("ORDER BY id"),
            OrderBy::Random => String::from("ORDER BY RANDOM()"),
            OrderBy::Name => String::from("ORDER BY name"),
        }
    } else {
        String::new()
    };

    // LIMIT
    let limit_clause = if let Some(Limit(limit)) = search.limit {
        format!("LIMIT {limit}")
    } else {
        String::new()
    };

    // SQL
    let sql = format!(
        r#"
            SELECT id AS "id: OpenTimelineId"
            FROM entities
            {where_clauses}
            {order_by_clause}
            {limit_clause}
        "#
    );

    let mut entity_ids: Vec<OpenTimelineId> = sqlx::query_scalar(&sql)
        .fetch_all(&mut **transaction)
        .await?;

    // Bool expr
    if let Some(bool_expr) = search.bool_expr.as_ref() {
        let table_info =
            bool_tag_expr::DbTableInfo::from("entity_tags", "entity_id", "name", "value").unwrap();

        let bool_expr_sql = bool_expr.clone().to_sql(&table_info);

        let sql = format!(
            r#"
                SELECT DISTINCT entity_id  AS "entity_id: OpenTimelineId"
                FROM ({bool_expr_sql})
                {limit_clause}
            "#
        );

        let bool_expr_entity_ids: Vec<OpenTimelineId> = sqlx::query_scalar(&sql)
            .fetch_all(&mut **transaction)
            .await?;

        entity_ids.extend(bool_expr_entity_ids);
    }

    // Remove any duplicates
    let entity_ids: BTreeSet<OpenTimelineId> = entity_ids.into_iter().collect();

    Ok(entity_ids.into_iter().collect())
}

/// Used to indicate start or end data
enum StartOrEnd {
    Start,
    End,
}

/// Create SQL to filter by date
fn create_date_cmp_sql(start_or_end: StartOrEnd, date_filter: &DateFilter) -> String {
    let ordering = date_filter.ordering;
    let start_or_end = match start_or_end {
        StartOrEnd::Start => "start",
        StartOrEnd::End => "end",
    };

    let year = date_filter.date.year();
    let month = date_filter.date.month();
    let day = date_filter.date.day();

    match (month, day) {
        (None, None) => {
            format!("{start_or_end}_year {ordering} {year}")
        }
        (Some(month), None) => {
            format!(
                "({start_or_end}_year * 100 + {start_or_end}_month) {ordering} ({year} * 100 + {month})"
            )
        }
        (Some(month), Some(day)) => {
            format!(
                "({start_or_end}_year * 10000 + {start_or_end}_month * 100 + {start_or_end}_day) {ordering} ({year} * 10000 + {month} * 100 + {day})"
            )
        }
        _ => String::new(),
    }
}
