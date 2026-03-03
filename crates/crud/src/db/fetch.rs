// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Fetch/read entities & timelines from a database or files
//!

use crate::crud::FetchById;
use crate::{BackupRestoreMergeError, db::open_file_for_reading};
use log::warn;
use open_timeline_core::{Entity, OpenTimelineId, TimelineEdit};
use sqlx::{Sqlite, Transaction};
use std::path::PathBuf;

/// Read timelines from a file
pub(super) async fn read_timelines_from_file(
    path: PathBuf,
) -> Result<Vec<TimelineEdit>, BackupRestoreMergeError> {
    let metadata = std::fs::metadata(path.clone()).map_err(BackupRestoreMergeError::StdIo)?;
    if metadata.len() == 0 {
        warn!("No timelines to restore: {path:?} is empty");
        return Ok(Vec::new());
    }
    let reader = open_file_for_reading(path.to_str().unwrap().to_string()).await?;
    Ok(serde_json::from_reader(reader).map_err(BackupRestoreMergeError::SerdeJson)?)
}

/// Read entities from a file
pub(super) async fn read_entities_from_file(
    path: PathBuf,
) -> Result<Vec<Entity>, BackupRestoreMergeError> {
    let metadata = std::fs::metadata(path.clone()).map_err(BackupRestoreMergeError::StdIo)?;
    if metadata.len() == 0 {
        warn!("No entities to restore: {path:?} is empty");
        return Ok(Vec::new());
    }
    let reader = open_file_for_reading(path.to_str().unwrap().to_string()).await?;
    Ok(serde_json::from_reader(reader).unwrap())
}

/// Read all entities from the database
pub(super) async fn fetch_all_entities_from_database(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<Vec<Entity>, BackupRestoreMergeError> {
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
    let mut all_entities = Vec::new();
    for id in ids {
        all_entities.push(
            Entity::fetch_by_id(transaction, &id)
                .await
                .map_err(BackupRestoreMergeError::CrudError)?,
        );
    }
    Ok(all_entities)
}

/// Read all timelines from the database
pub(super) async fn fetch_all_timelines_from_database(
    transaction: &mut Transaction<'_, Sqlite>,
) -> Result<Vec<TimelineEdit>, BackupRestoreMergeError> {
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
    let mut backup_timelines = Vec::new();
    for id in ids {
        let timeline = TimelineEdit::fetch_by_id(transaction, &id).await.unwrap();
        backup_timelines.push(timeline);
    }
    Ok(backup_timelines)
}
