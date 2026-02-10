// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! *Part of the wider OpenTimeline project*
//!
//! This library crate is responsible for all database interactions and management
//! for the OpenTimeline project.  It does the following:
//!
//! - Enables CRUD (Create, Read, Update, Delete) operations on entities
//! - Provides types & functionality for editing & viewing timelines
//! - Provides search functionality for entities and timelines by both text
//! (name) and boolean expressions of tags.
//! - Provides helpers to get table row counts
//! - Provides helpers to get information about the number of entities,
//! timelines, tags, subtimelines.
//! - Enables bulk tag editing opeation
//!
//! This crate makes use of the basic OpenTimeline `core` crate for primitive
//! types, and is itself used by the `api` and `gui` crates.
//!

mod backup;
mod crud;
mod db;
mod stats;

pub use backup::*;
pub use crud::*;
pub use db::*;
pub use stats::*;

use serde::{Deserialize, Serialize};

/// How to sort a collection of numbers
#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum SortByNumber {
    Ascending,
    Descending,
}

/// How to sort a collection of strings
#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum SortAlphabetically {
    AToZ,
    ZToA,
}

#[cfg(test)]
pub mod test {
    use crate::{Create, restore};
    use open_timeline_core::{Entity, TimelineEdit};
    use sqlx::{Sqlite, Transaction};
    use std::fs::File;
    use std::io;
    use std::io::BufRead;
    use std::path::PathBuf;

    pub fn path_to_test_data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data")
    }

    pub async fn seed_db(transaction: &mut Transaction<'_, Sqlite>) {
        // Seed the database
        let dir = path_to_test_data().join("seed");
        restore(transaction, dir.clone()).await.unwrap();
    }

    pub async fn seed_db_return_timelines(
        transaction: &mut Transaction<'_, Sqlite>,
    ) -> Vec<TimelineEdit> {
        // Seed the database
        let dir = path_to_test_data().join("seed");
        restore(transaction, dir.clone()).await.unwrap();

        // Get the timelines that seeded the database
        valid_timelines()
    }

    pub async fn seed_db_with_entities(transaction: &mut Transaction<'_, Sqlite>) {
        for mut entity in valid_entities() {
            entity.create(transaction).await.unwrap();
        }
    }

    pub fn valid_timelines() -> Vec<TimelineEdit> {
        let path = path_to_test_data().join("seed/timelines.json");
        let file = File::open(path).unwrap();
        let timelines: Vec<TimelineEdit> = serde_json::from_reader(file).unwrap();
        timelines
    }

    pub fn valid_timelines_no_subtimelines() -> Vec<TimelineEdit> {
        valid_timelines()
            .into_iter()
            .map(|mut timeline| {
                timeline.clear_subtimelines();
                timeline
            })
            .collect()
    }

    pub fn valid_timeline_no_subtimelines() -> TimelineEdit {
        let mut timeline = valid_timelines().pop().unwrap();
        timeline.clear_subtimelines();
        timeline
    }

    pub fn valid_timeline_with_bool_expr() -> TimelineEdit {
        for timeline in valid_timelines() {
            if timeline.bool_expr().is_some() {
                return timeline;
            }
        }
        panic!()
    }

    pub fn valid_entities() -> Vec<Entity> {
        let file = path_to_test_data().join("entities/valid/1.json");
        let json_string = load_jsonc_strip_leading_comment_lines(&file);
        let entities: Vec<Entity> = serde_json::from_str(&json_string).unwrap();
        entities
    }

    pub fn valid_entity() -> Entity {
        let entities = valid_entities();
        let entity = entities.first().unwrap().clone();
        entity
    }

    pub fn load_jsonc_strip_leading_comment_lines(path: &PathBuf) -> String {
        // Open the file for reading
        let file = File::open(path).unwrap();
        let reader = io::BufReader::new(file);

        // Holds the JSON as it's collected
        let mut json_content = String::new();

        // Collect all lines that don't begin with "//"
        for line in reader.lines() {
            let line = line.unwrap();
            if !line.starts_with("//") {
                json_content.push_str(&line);
                json_content.push('\n');
            }
        }

        // Return the JSON now that the comment(s) at the top of the file have
        // been removed
        json_content
    }
}
