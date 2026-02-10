// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Stats
//!

use crate::CrudError;
use sqlx::Row;
use sqlx::Sqlite;
use sqlx::Transaction;

/// Each variant maps to a table in the database
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Table {
    /// Represents the `entities` table
    Entities,

    /// Represents the `entity_tags` table
    EntityTags,

    /// Represents the `timelines` table
    Timelines,

    /// Represents the `subtimelines` table
    Subtimelines,

    /// Represents the `timeline_entities` table
    TimelineEntities,

    /// Represents the `timeline_tags` table
    TimelineTags,
}

/// Holds database row counts
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DatabaseRowCount {
    /// The number of rows in the `entities` table
    pub entities: i64,

    /// The number of rows in the `entity_tags` table
    pub entity_tags: i64,

    /// The number of rows in the `timelines` table
    pub timelines: i64,

    /// The number of rows in the `subtimelines` table
    pub subtimelines: i64,

    /// The number of rows in the `timeline_entities` table
    pub timeline_entities: i64,

    /// The number of rows in the `timeline_tags` table
    pub timeline_tags: i64,
}

impl DatabaseRowCount {
    // TODO could store futures for lazy counting (ie remove the awaits)
    /// Fetch the row count for all tables in the database
    pub async fn all(transaction: &mut Transaction<'_, Sqlite>) -> Result<Self, CrudError> {
        Ok(Self {
            entities: Self::table(transaction, Table::Entities).await?,
            entity_tags: Self::table(transaction, Table::EntityTags).await?,
            timelines: Self::table(transaction, Table::Timelines).await?,
            subtimelines: Self::table(transaction, Table::Subtimelines).await?,
            timeline_entities: Self::table(transaction, Table::TimelineEntities).await?,
            timeline_tags: Self::table(transaction, Table::TimelineTags).await?,
        })
    }

    /// Get the row count for a specific table in the database
    pub async fn table(
        transaction: &mut Transaction<'_, Sqlite>,
        table_name: Table,
    ) -> Result<i64, CrudError> {
        let table_name = match table_name {
            Table::Entities => "entities",
            Table::EntityTags => "entity_tags",
            Table::Timelines => "timelines",
            Table::Subtimelines => "subtimelines",
            Table::TimelineEntities => "timeline_entities",
            Table::TimelineTags => "timeline_tags",
        };

        let row = sqlx::query(&format!("SELECT COUNT(*) AS row_count FROM {table_name}"))
            .fetch_one(&mut **transaction)
            .await?;
        Ok(row.get("row_count"))
    }
}
