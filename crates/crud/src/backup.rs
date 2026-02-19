// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Backup, restore & merge all entities and timelines to & from JSON
//!

use crate::crud::{Create, CrudError, FetchById, Update};
use crate::{is_entity_id_in_db, is_timeline_id_in_db};
use log::warn;
use open_timeline_core::{Entity, HasIdAndName, OpenTimelineId, TimelineEdit};
use sqlx::{Sqlite, Transaction};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Possible operations & used to indicate success
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BackupMergeRestore {
    /// Used to indicate a we want to backup timelines & entities
    Backup,

    /// Used to indicate we want to merge in timelines & entities
    Merge,

    /// Used to indicate we want to restore the database
    Restore,
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

/// Backup the database to JSON
pub async fn backup(
    transaction: &mut Transaction<'_, Sqlite>,
    backup_dir_path: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    backup_or_restore_or_merge(transaction, backup_dir_path, BackupMergeRestore::Backup).await
}

/// Merge the database to JSON
pub async fn merge(
    transaction: &mut Transaction<'_, Sqlite>,
    merge_dir_path: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    backup_or_restore_or_merge(transaction, merge_dir_path, BackupMergeRestore::Merge).await
}

/// Restore the database to JSON
pub async fn restore(
    transaction: &mut Transaction<'_, Sqlite>,
    restore_dir_path: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    backup_or_restore_or_merge(transaction, restore_dir_path, BackupMergeRestore::Restore).await
}

/// Backup, merge, or restore a database
async fn backup_or_restore_or_merge(
    transaction: &mut Transaction<'_, Sqlite>,
    backup_dir_path: PathBuf,
    backup_merge_restore: BackupMergeRestore,
) -> Result<(), BackupRestoreMergeError> {
    match backup_merge_restore {
        BackupMergeRestore::Backup => {
            backup_entities(transaction, backup_dir_path.clone()).await?;
            backup_timelines(transaction, backup_dir_path.clone()).await?;
        }
        BackupMergeRestore::Merge => {
            merge_entities(transaction, backup_dir_path.clone()).await?;
            merge_timelines(transaction, backup_dir_path.clone()).await?;
        }
        BackupMergeRestore::Restore => {
            clear_db(transaction).await?;
            merge_entities(transaction, backup_dir_path.clone()).await?;
            merge_timelines(transaction, backup_dir_path.clone()).await?;
        }
    }
    Ok(())
}

/// Backup entities in the database to JSON
async fn backup_entities(
    transaction: &mut Transaction<'_, Sqlite>,
    mut backup_dir: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    // Get all entity IDs
    let ids: Vec<OpenTimelineId> = sqlx::query_scalar!(
        r#"
            SELECT id AS "id: OpenTimelineId"
            FROM entities
        "#
    )
    .fetch_all(&mut **transaction)
    .await
    .map_err(BackupRestoreMergeError::Sqlx)?;

    // Get all entities from their ID
    let mut all_entities: Vec<Entity> = vec![];
    for id in ids {
        all_entities.push(
            Entity::fetch_by_id(transaction, &id)
                .await
                .map_err(BackupRestoreMergeError::CrudError)?,
        );
    }

    // Convert the list of entities to JSON and save it to the `entities.json`
    // file
    let json =
        serde_json::to_string_pretty(&all_entities).map_err(BackupRestoreMergeError::SerdeJson)?;
    backup_dir.push("entities.json");
    create_and_write_to_file(&backup_dir, json).await?;

    Ok(())
}

/// Backup timelines in the database to JSON
async fn backup_timelines(
    transaction: &mut Transaction<'_, Sqlite>,
    mut backup_dir: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    // Get all timeline IDs
    let ids: Vec<OpenTimelineId> = sqlx::query_scalar!(
        r#"
            SELECT id AS "id: OpenTimelineId"
            FROM timelines
        "#
    )
    .fetch_all(&mut **transaction)
    .await
    .map_err(BackupRestoreMergeError::Sqlx)?;

    // Get all timelines from their ID
    let mut backup_timelines: Vec<TimelineEdit> = Vec::new();
    for id in ids {
        let timeline = TimelineEdit::fetch_by_id(transaction, &id).await.unwrap();
        backup_timelines.push(timeline);
    }

    // Convert the list of timelines to JSON and save it to the `timeline.json`
    // file
    let json = serde_json::to_string_pretty(&backup_timelines).unwrap();
    backup_dir.push("timelines.json");
    create_and_write_to_file(&backup_dir, json).await?;

    Ok(())
}

// TODO: call `tx.rollback().await?;` if error?
/// Merge entities from backup.
///
/// Every entity to be merged in must have an ID, else an error is returned.  If
/// the entity ID is already in the database, the existing entity is replaced by
/// the incoming entity.  If the entity ID is not already in the database, the
/// incoming entity is inserted.
async fn merge_entities(
    transaction: &mut Transaction<'_, Sqlite>,
    mut backup_dir: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    backup_dir.push("entities.json");
    let reader = open_file_for_reading(backup_dir.to_str().unwrap().to_string()).await?;
    let entities: Vec<Entity> = serde_json::from_reader(reader).unwrap();
    for mut entity in entities {
        // The entity must have an ID
        let entity_id = entity
            .id()
            .ok_or(CrudError::IdNotSetForEntity(entity.name().to_owned()))?;

        // If the entity ID is already in the database, the update the entity,
        // otherwise create it
        match is_entity_id_in_db(transaction, &entity_id).await? {
            true => entity.update(transaction).await,
            false => entity.create(transaction).await,
        }
        .map_err(BackupRestoreMergeError::CrudError)?;
    }
    Ok(())
}

/// Merge timelines from backup.
///
/// Every timeline to be merged in must have an ID, else an error is returned.
/// If the timeline ID is already in the database, the existing timeline is
/// replaced by the incoming timeline.  If the timeline ID is not already in
/// the database, the incoming timeline is inserted.
async fn merge_timelines(
    transaction: &mut Transaction<'_, Sqlite>,
    mut backup_dir: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    backup_dir.push("timelines.json");

    // TODO (do for restore_entities too) (keep?)
    let metadata = std::fs::metadata(backup_dir.clone()).map_err(BackupRestoreMergeError::StdIo)?;
    if metadata.len() == 0 {
        warn!("No timelines to restore: {backup_dir:?} is empty");
        return Ok(());
    }

    let reader = open_file_for_reading(backup_dir.to_str().unwrap().to_string()).await?;
    let backup_timelines: Vec<TimelineEdit> =
        serde_json::from_reader(reader).map_err(BackupRestoreMergeError::SerdeJson)?;

    // Insert timelines without subtimelines (FOREIGN KEYs would fail otherwise)
    for mut timeline in backup_timelines.clone() {
        timeline.clear_subtimelines();

        // The timeline must have an ID
        let timeline_id = timeline
            .id()
            .ok_or(CrudError::IdNotSetForEntity(timeline.name().to_owned()))?;

        // If the timeline ID is already in the database, the update the timeline,
        // otherwise create it (without subtimelines)
        match is_timeline_id_in_db(transaction, &timeline_id).await? {
            true => timeline.update(transaction).await,
            false => timeline.create(transaction).await,
        }
        .map_err(BackupRestoreMergeError::CrudError)?;
    }

    // Update timelines to save their subtimelines
    for mut timeline in backup_timelines {
        timeline
            .update(transaction)
            .await
            .map_err(BackupRestoreMergeError::CrudError)?;
    }

    Ok(())
}

/// Clear the database
async fn clear_db(
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
        restore(&mut transaction, seed_dir_to_restore_from.clone())
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
        backup(&mut transaction, new_dir.clone()).await.unwrap();

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
