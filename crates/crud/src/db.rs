// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Manage OpenTimeline databases
//!
//! - Backup, merge & restore all entities and timelines to & from JSON files
//! - Merge & restore entities and timelines from lists in memory
//! - Create a new database
//! - Clear a database
//!

mod backup;
mod fetch;
mod merge;

use backup::*;
use fetch::*;
use merge::*;
use sqlx::Pool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::crud::CrudError;
use log::info;
use open_timeline_core::{Entity, TimelineEdit};
use sqlx::{Sqlite, SqlitePool, Transaction, migrate::MigrateDatabase};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

/// The OpenTimeline database
#[derive(Debug)]
pub struct OpenTimelineDatabase {}

impl OpenTimelineDatabase {
    /// Create a URL for the SQLite database using the path to the database
    pub fn url_from_path(path: &Path) -> String {
        format!("sqlite://{}", path.to_string_lossy())
    }

    /// Setup a database at the supplied path (ensure the file exists and run the
    /// migrations
    pub async fn setup_at_path(path: &Path) -> Result<(), sqlx::Error> {
        // Construct the database URL
        let db_url = Self::url_from_path(path);

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create the database file (if not already extant)
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            info!("Creating database at {}", path.to_string_lossy());
            Sqlite::create_database(&db_url).await?;
        } else {
            info!("Database already exists at {}", path.to_string_lossy());
        }

        // Open a connection
        let pool = SqlitePool::connect(&db_url).await?;

        // Run migrations (uses compile-time embedding of migrations)
        Self::migrate_pool(&pool).await?;

        info!(
            "Migrations applied successfully to {}",
            path.to_string_lossy()
        );

        Ok(())
    }

    // TODO: test the read-only aspect?
    ///
    pub async fn sqlite_pool_from_url(
        db_url: &str,
        read_only: bool,
    ) -> Result<Pool<Sqlite>, sqlx::Error> {
        // Create connection options (whether the database is read-only or not)
        let connect_options = SqliteConnectOptions::from_str(db_url)?.read_only(read_only);

        // Create a pool with those options
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(connect_options)
            .await
    }

    /// Run migrations (uses compile-time embedding of migrations)
    pub async fn migrate_pool(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        Ok(sqlx::migrate!("./migrations").run(pool).await?)
    }

    /// Clear the database
    async fn clear(
        transaction: &mut Transaction<'_, Sqlite>,
    ) -> Result<(), BackupRestoreMergeError> {
        let mut queries = Vec::new();

        // Create the queries (order important because of FOREIGN KEY constraints)
        queries.push(sqlx::query!("DELETE FROM timeline_tags;"));
        queries.push(sqlx::query!("DELETE FROM timeline_entities;"));
        queries.push(sqlx::query!("DELETE FROM subtimelines;"));
        queries.push(sqlx::query!("DELETE FROM timelines;"));
        queries.push(sqlx::query!("DELETE FROM entity_tags;"));
        queries.push(sqlx::query!("DELETE FROM entities;"));

        // Execute all the DELETE queries (not committed)
        for query in queries {
            query
                .execute(&mut **transaction)
                .await
                .map_err(BackupRestoreMergeError::Sqlx)?;
        }

        Ok(())
    }

    /// Backup the database to JSON files in the given dir
    pub async fn backup_to_dir(
        transaction: &mut Transaction<'_, Sqlite>,
        dir: PathBuf,
    ) -> Result<(), BackupRestoreMergeError> {
        backup_database_entities_to_dir(transaction, dir.clone()).await?;
        backup_database_timelines_to_dir(transaction, dir).await
    }

    /// Merge into the database the JSON files in the given dir
    pub async fn merge_from_dir(
        transaction: &mut Transaction<'_, Sqlite>,
        dir: PathBuf,
    ) -> Result<(), BackupRestoreMergeError> {
        merge_entities_from_dir(transaction, dir.clone()).await?;
        merge_timelines_from_dir(transaction, dir).await
    }

    /// Restore the database from the JSON files in the given dir
    pub async fn restore_from_dir(
        transaction: &mut Transaction<'_, Sqlite>,
        dir: PathBuf,
    ) -> Result<(), BackupRestoreMergeError> {
        Self::clear(transaction).await?;
        Self::merge_from_dir(transaction, dir).await
    }

    /// Merge into the database the given data
    pub async fn merge_from_data(
        transaction: &mut Transaction<'_, Sqlite>,
        entities: Vec<Entity>,
        timelines: Vec<TimelineEdit>,
    ) -> Result<(), BackupRestoreMergeError> {
        merge_entities(transaction, entities).await?;
        merge_timelines(transaction, timelines).await
    }

    /// Restore the database from the given data
    pub async fn restore_from_data(
        transaction: &mut Transaction<'_, Sqlite>,
        entities: Vec<Entity>,
        timelines: Vec<TimelineEdit>,
    ) -> Result<(), BackupRestoreMergeError> {
        Self::clear(transaction).await?;
        Self::merge_from_data(transaction, entities, timelines).await
    }
}

/// Errors that can occur when backing up/merging in/restoring OpenTimeline.
/// databases
#[derive(Debug, Error)]
pub enum BackupRestoreMergeError {
    /// A CRUD error occurred
    #[error(transparent)]
    CrudError(#[from] CrudError),

    /// An error occured when working with the backup/merge/restore dir or files.
    #[error(transparent)]
    StdIo(#[from] std::io::Error),

    /// A database error occured in this module (database errors else where will.
    /// be returned as a `CrudError`)
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// A JSON error occured (most likely when reading a JSON file).
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// An error when fetching from a web API.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

/// Open the file in read-only mode and return the buffer
async fn open_file_for_reading(
    path_string: String,
) -> Result<BufReader<File>, BackupRestoreMergeError> {
    let path = Path::new(&path_string);
    let file = File::open(path).map_err(BackupRestoreMergeError::StdIo)?;
    Ok(BufReader::new(file))
}

/// Write a string to file at some path
async fn create_and_write_to_file(
    path: &Path,
    content: String,
) -> Result<(), BackupRestoreMergeError> {
    let mut file = File::create(path).map_err(BackupRestoreMergeError::StdIo)?;
    file.write_all(content.as_bytes())
        .map_err(BackupRestoreMergeError::StdIo)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{DatabaseRowCount, test::path_to_test_data};
    use open_timeline_core::OpenTimelineId;
    use sqlx::{Pool, Sqlite};
    use std::{fs, path::PathBuf};

    // TODO: use tempdir crate
    #[sqlx::test]
    fn backup_restore_merge(pool: Pool<Sqlite>) {
        // Setup
        let mut transaction = pool.begin().await.unwrap();

        // Setup files to restore from (create a new dir in /tmp)
        let seed_dir_to_restore_from = path_to_test_data().join("seed");
        let original_entities_path = &seed_dir_to_restore_from.join("entities.json");
        let original_timelines_path = &seed_dir_to_restore_from.join("timelines.json");

        // Restore from the dir
        OpenTimelineDatabase::restore_from_dir(&mut transaction, seed_dir_to_restore_from.clone())
            .await
            .unwrap();

        // Check the row counts
        let row_counts = DatabaseRowCount::all(&mut transaction).await.unwrap();
        assert_eq!(row_counts.entities, 3);
        assert_eq!(row_counts.entity_tags, 8);
        assert_eq!(row_counts.timelines, 2);
        assert_eq!(row_counts.subtimelines, 1);
        assert_eq!(row_counts.timeline_entities, 3);
        assert_eq!(row_counts.timeline_tags, 2);

        // Setup the new dir (create yet another new dir in /tmp)
        let new_dir = PathBuf::from(format!("/tmp/{}", OpenTimelineId::new()));
        fs::create_dir(&new_dir).unwrap();
        let new_entities_path = &new_dir.join("entities.json");
        let new_timelines_path = &new_dir.join("timelines.json");

        // Backup from the database
        OpenTimelineDatabase::backup_to_dir(&mut transaction, new_dir.clone())
            .await
            .unwrap();

        // Get original JSON (that we restored from)
        let original_entities = fs::read(original_entities_path).unwrap();
        let original_timelines = fs::read(original_timelines_path).unwrap();

        // Get new JSON (that we created when backing up)
        let new_entities = fs::read(new_entities_path).unwrap();
        let new_timelines = fs::read(new_timelines_path).unwrap();

        // Delete the new tmp dirs
        fs::remove_dir_all(new_dir).unwrap();

        // Check the backup JSON is identical to the JSON restored from
        assert_eq!(original_entities, new_entities);
        assert_eq!(original_timelines, new_timelines);
    }
}
