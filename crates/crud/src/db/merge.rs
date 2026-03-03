// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Merge in entities & timelines supplied or fetched from files
//!

use crate::crud::{Create, CrudError, Update};
use crate::db::{read_entities_from_file, read_timelines_from_file};
use crate::{BackupRestoreMergeError, is_entity_id_in_db, is_timeline_id_in_db};
use open_timeline_core::{Entity, HasIdAndName, TimelineEdit};
use sqlx::{Sqlite, Transaction};
use std::path::PathBuf;

/// Merge the given timelines into the database.
///
/// Every timeline to be merged in must have an ID, else an error is returned.
/// If the timeline ID is already in the database, the existing timeline is
/// replaced by the incoming timeline.  If the timeline ID is not already in
/// the database, the incoming timeline is inserted.
pub(super) async fn merge_timelines(
    transaction: &mut Transaction<'_, Sqlite>,
    timelines: Vec<TimelineEdit>,
) -> Result<(), BackupRestoreMergeError> {
    // Insert timelines without subtimelines (FOREIGN KEYs would fail otherwise)
    for mut timeline in timelines.clone() {
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
    for mut timeline in timelines {
        timeline
            .update(transaction)
            .await
            .map_err(BackupRestoreMergeError::CrudError)?;
    }
    Ok(())
}

/// Merge the given entities into the database.
///
/// Every entity to be merged in must have an ID, else an error is returned.  If
/// the entity ID is already in the database, the existing entity is replaced by
/// the incoming entity.  If the entity ID is not already in the database, the
/// incoming entity is inserted.
pub(super) async fn merge_entities(
    transaction: &mut Transaction<'_, Sqlite>,
    entities: Vec<Entity>,
) -> Result<(), BackupRestoreMergeError> {
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

/// Merge entities in a JSON file in the given dir into the database.
pub(super) async fn merge_entities_from_dir(
    transaction: &mut Transaction<'_, Sqlite>,
    mut backup_dir: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    backup_dir.push("entities.json");
    let entities = read_entities_from_file(backup_dir).await?;
    merge_entities(transaction, entities).await
}

/// Merge timelines in a JSON file in the given dir into the database.
pub(super) async fn merge_timelines_from_dir(
    transaction: &mut Transaction<'_, Sqlite>,
    mut backup_dir: PathBuf,
) -> Result<(), BackupRestoreMergeError> {
    backup_dir.push("timelines.json");
    let timelines = read_timelines_from_file(backup_dir).await?;
    merge_timelines(transaction, timelines).await
}
