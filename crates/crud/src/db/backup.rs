// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Backup entities & timelines to a dir
//!

use crate::BackupRestoreMergeError;
use crate::db::{
    create_and_write_to_file, fetch_all_entities_from_database, fetch_all_timelines_from_database,
};
use sqlx::{Sqlite, Transaction};
use std::path::PathBuf;

/// Backup entities in the database to JSON
pub(super) async fn backup_database_entities_to_dir(
    transaction: &mut Transaction<'_, Sqlite>,
    mut backup_dir: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    let all_entities = fetch_all_entities_from_database(transaction).await?;

    // Convert the list of entities to JSON and save it to the `entities.json`
    // file
    let json =
        serde_json::to_string_pretty(&all_entities).map_err(BackupRestoreMergeError::SerdeJson)?;
    backup_dir.push("entities.json");
    create_and_write_to_file(&backup_dir, json).await
}

/// Backup timelines in the database to JSON
pub(super) async fn backup_database_timelines_to_dir(
    transaction: &mut Transaction<'_, Sqlite>,
    mut backup_dir: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    let backup_timelines = fetch_all_timelines_from_database(transaction).await?;

    // Convert the list of timelines to JSON and save it to the `timeline.json`
    // file
    let json = serde_json::to_string_pretty(&backup_timelines).unwrap();
    backup_dir.push("timelines.json");
    create_and_write_to_file(&backup_dir, json).await
}
