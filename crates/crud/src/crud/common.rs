// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! CRUD traits and errors
//!

use async_trait::async_trait;
use bool_tag_expr::{BoolTagExpr, ParseError, Tag};
use open_timeline_core::{
    IsReducedType, Name, OpenTimelineId, ReducedEntities, ReducedTimeline, ReducedTimelines,
};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use thiserror::Error;

/// Alias of u64
pub type RowsAffected = u64;

/// A collection of reduced timelines & reduced entities.
#[derive(Debug, Clone, Hash, PartialEq, Serialize, Eq)]
pub struct ReducedAll {
    entities: ReducedEntities,
    timelines: ReducedTimelines,
}

// TODO: impl fetch from partial name and tags?
impl ReducedAll {
    /// Create a new `ReducedAll`
    pub fn new(entities: ReducedEntities, timelines: ReducedTimelines) -> Self {
        Self {
            entities,
            timelines,
        }
    }

    /// Get the `ReducedEntities`
    pub fn entities(&self) -> &ReducedEntities {
        &self.entities
    }

    /// Get the `ReducedTimelines`
    pub fn timelines(&self) -> &ReducedTimelines {
        &self.timelines
    }
}

#[async_trait]
impl FetchAllWithTag for ReducedAll {
    /// Get all things (entities & timelines) that have the given tag.
    ///
    /// For more complicated tag-related queries, use the boolean expression
    /// functionality.
    async fn fetch_all_with_tag(
        transaction: &mut Transaction<'_, Sqlite>,
        tag: &Tag,
    ) -> Result<ReducedAll, CrudError> {
        let timelines = ReducedTimelines::fetch_all_with_tag(transaction, tag).await?;
        let entities = ReducedEntities::fetch_all_with_tag(transaction, tag).await?;
        Ok(ReducedAll::new(entities, timelines))
    }
}

/// Hold an ID or a Name.  Used to indicate whether a string is an ID or name.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum IdOrName {
    /// Holds an ID
    Id(OpenTimelineId),

    /// Holds a name
    Name(Name),
}

/// Used to limit the number of things fetched/returned.
///
/// Can easily be destructured, e.g.:
///
/// ```
/// use open_timeline_crud::Limit;
///
/// fn my_func(Limit(limit): Limit) {
///     println!("Limit is {}", limit);
/// }
/// ```
#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone)]
pub struct Limit(pub u32);

/// Implementing types can fetch all instances
#[allow(async_fn_in_trait)]
#[async_trait]
pub trait FetchAll: Sized + Send {
    async fn fetch_all(transaction: &mut Transaction<'_, Sqlite>) -> Result<Self, CrudError>;
}

/// Implementing types can fetch all instances that have some tag
///
/// For more complicated tag-related queries, use the boolean expression
/// functionality.
#[allow(async_fn_in_trait)]
#[async_trait]
pub trait FetchAllWithTag: Sized + Send {
    async fn fetch_all_with_tag(
        transaction: &mut Transaction<'_, Sqlite>,
        tag: &Tag,
    ) -> Result<Self, CrudError>;
}

/// Implementing types can be fetched using both a partial name and a boolean
/// tag expression
#[allow(async_fn_in_trait)]
#[async_trait]
pub trait FetchByPartialNameAndBoolTagExpr:
    FetchByBoolTagExpr + FetchByPartialName + Sized + Send
{
    /// Fetch the thing using a partial name and a boolean tag expression
    async fn fetch_by_partial_name_and_bool_tag_expr(
        transaction: &mut Transaction<'_, Sqlite>,
        limit: Limit,
        partial_name: &str,
        bool_tag_expr: BoolTagExpr,
    ) -> Result<Self, CrudError>;
}

/// Implementing types can be fetched using a boolean tag expression
#[allow(async_fn_in_trait)]
#[async_trait]
pub trait FetchByBoolTagExpr: Sized + Send {
    /// Fetch the thing using a boolean tag expression
    async fn fetch_by_bool_tag_expr(
        transaction: &mut Transaction<'_, Sqlite>,
        limit: Limit,
        bool_tag_expr: BoolTagExpr,
    ) -> Result<Self, CrudError>;
}

/// Implementing types can be fetched using a partial name (which is a `String`
/// rather than a [`Name`] because the partial name can be empty)
#[allow(async_fn_in_trait)]
#[async_trait]
pub trait FetchByPartialName: Sized + Send {
    /// Fetch the thing using a partial name
    async fn fetch_by_partial_name(
        transaction: &mut Transaction<'_, Sqlite>,
        limit: Limit,
        partial_name: &str,
    ) -> Result<Self, CrudError>;
}

/// Implementing types can be fetched using their [`OpenTimelineId`]
#[allow(async_fn_in_trait)]
pub trait FetchById: Sized {
    /// Fetch the thing using its [`OpenTimelineId`]
    async fn fetch_by_id(
        transaction: &mut Transaction<'_, Sqlite>,
        id: &OpenTimelineId,
    ) -> Result<Self, CrudError>;
}

/// Implementing types can be fetched using their [`Name`]
#[allow(async_fn_in_trait)]
pub trait FetchByName: Sized {
    /// Fetch the thing using its [`Name`]
    async fn fetch_by_name(
        transaction: &mut Transaction<'_, Sqlite>,
        name: &Name,
    ) -> Result<Self, CrudError>;
}

/// Implementing types can deleted using their [`Name`]
#[allow(async_fn_in_trait)]
pub trait DeleteByName {
    /// Delete the thing using its [`Name`]
    async fn delete_by_name(
        transaction: &mut Transaction<'_, Sqlite>,
        name: &Name,
    ) -> Result<(), CrudError>;
}

/// Implementing types can deleted using their [`OpenTimelineId`]
#[allow(async_fn_in_trait)]
pub trait DeleteById {
    /// Delete the thing using its [`OpenTimelineId`]
    async fn delete_by_id(
        transaction: &mut Transaction<'_, Sqlite>,
        id: &OpenTimelineId,
    ) -> Result<(), CrudError>;
}

/// Implementing types can be created in the database
#[allow(async_fn_in_trait)]
pub trait Create {
    /// Create the data in the database
    async fn create(&mut self, transaction: &mut Transaction<'_, Sqlite>) -> Result<(), CrudError>;
}

/// Implementing types can be updated in the database
#[allow(async_fn_in_trait)]
pub trait Update {
    async fn update(&mut self, transaction: &mut Transaction<'_, Sqlite>) -> Result<(), CrudError>;
}

// TODO: crush database errors into one (connection, etc, not missing from DB)
/// All errors that could occur when running CRUD operations
#[derive(Debug, Error, Clone, Hash, PartialEq, Eq)]
pub enum CrudError {
    #[error("The name field is not set")]
    NameNotSet,

    #[error("{0}")]
    BoolExprParse(ParseError),

    // TODO: really should impl From<NameError> for CrudError
    #[error("Name error")]
    Name,

    // TODO: really should impl From<DateError> for CrudError
    #[error("Date error")]
    Date,

    #[error("The ID field is not set for entity '{0}'")]
    IdNotSetForEntity(Name),

    #[error("The ID field is not set for timeline '{0}'")]
    IdNotSetForTimeline(Name),

    #[error("The entity's ID is already in use")]
    EntityIdAlreadyInUse,

    #[error("The entity's name ('{0}') is already in use")]
    EntityNameAlreadyInUse(Name),

    #[error("The entity's start year is not set")]
    EntityStartYearNotSet,

    #[error("Neither the ID nor the name is set")]
    NeitherIdNorNameSet,

    #[error("Both the ID and name are set")]
    BothIdAndNameSet,

    #[error("No database file selected")]
    NoDbSelected,

    #[error("Not unique in the database: {0}")]
    NotUniqueInDb(String),

    #[error("There was a error with the database")]
    DbError,

    #[error("Error when trying to establish a new connection to the database")]
    DbNewConnection,

    #[error("Error when trying to establish a new transaction with the database")]
    DbNewTransaction,

    #[error("SQLx database error: {0}")]
    SqlxDbError(String),

    #[error("The ID is not in the database")]
    IdNotInDb,

    #[error("The name is not in the database")]
    NameNotInDb,

    #[error("Not in the database")]
    NotInDb,

    #[error("The timeline is not in the database")]
    TimelineNotInDb,

    #[error("The entity is not in the database")]
    EntityNotInDb,

    #[error("Error when updating the name")]
    UpdatingName,

    #[error("Error when fetching the timeline's direct member entities")]
    FetchingTimelineDirectMemberEntities,

    #[error("Error when fetching the timeline's direct subtimeline IDs")]
    FetchingTimelineDirectSubtimelineIds,

    #[error("Error when fetching the IDs of the timeline's entities")]
    FetchingTimelineAllEntityIds,

    #[error("Error when fetching the timeline's tags")]
    FetchingTimelineTags,

    #[error("It is neither an Id nor a Name")]
    NeitherIdNorName,

    #[error("IO error: {0}")]
    Io(String),

    #[error("JSON error: {0}")]
    Json(String),

    // TODO: not really a CRUD error! (Add an OpenTimelineError)
    #[error("GUI config error")]
    Config,

    #[error("Database migration error: {0}")]
    DbMigrate(String),
}

impl From<sqlx::Error> for CrudError {
    // TODO: do this properly
    fn from(value: sqlx::Error) -> Self {
        if let Some(db_err) = value.as_database_error() {
            if db_err.is_unique_violation() {
                // db_err.constraint().unwrap(), // TODO: this is only supported by PostGres driver, according to the crate source
                return CrudError::NotUniqueInDb(db_err.message().to_string());
            }
        }

        Self::SqlxDbError(value.to_string())
    }
}

impl From<std::io::Error> for CrudError {
    fn from(value: std::io::Error) -> Self {
        CrudError::Io(value.to_string())
    }
}

impl From<serde_json::Error> for CrudError {
    fn from(value: serde_json::Error) -> Self {
        CrudError::Json(value.to_string())
    }
}

// TODO: needs testing
/// Whether the given string is an [`OpenTimelineId`] or [`Name`] or neither
pub fn string_is_name_or_id(id_or_name: String) -> Option<IdOrName> {
    match (OpenTimelineId::from(&id_or_name), Name::from(&id_or_name)) {
        (Ok(id), _) => Some(IdOrName::Id(id)),
        (Err(_), Ok(name)) => Some(IdOrName::Name(name)),
        _ => None,
    }
}

/// Fetch the timelines that the given entity is a direct member of
pub async fn fetch_timelines_that_entity_is_direct_member_of(
    transaction: &mut Transaction<'_, Sqlite>,
    entity_id: &OpenTimelineId,
) -> Result<ReducedTimelines, CrudError> {
    Ok(sqlx::query!(
        r#"
            SELECT
                timeline_entities.timeline_id AS "timeline_id: OpenTimelineId",
                timelines.name AS "timeline_name: Name"
            FROM timeline_entities
            JOIN timelines ON
                timeline_entities.timeline_id = timelines.id
            WHERE entity_id=?
        "#,
        entity_id
    )
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .map(|row| ReducedTimeline::from_id_and_name(row.timeline_id, row.timeline_name))
    .collect())
}
