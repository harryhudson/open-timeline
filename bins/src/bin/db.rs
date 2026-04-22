// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Simple database management CLI
//!
//! ## Usage
//!
//! - `cargo r --bin db`
//! - `cargo r --bin db -- create  --database /path/to/sqlite/database`
//! - `cargo r --bin db -- backup  --database /path/to/sqlite/database --json /path/to/json/data/`
//! - `cargo r --bin db -- merge   --database /path/to/sqlite/database --json /path/to/json/data/`
//! - `cargo r --bin db -- restore --database /path/to/sqlite/database --json /path/to/json/data/`
//! - `cargo r --bin db -- stats   --database /path/to/sqlite/database`
//!

use clap::{CommandFactory, Parser, ValueEnum, builder::PossibleValue};
use open_timeline_crud::{DatabaseRowCount, OpenTimelineDatabase};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    // Check the options
    match (&args.cli_command, &args.database, &args.json) {
        //----------------------------------------------------------------------
        // Valid
        //----------------------------------------------------------------------
        (Command::Create, database_path, _) => {
            let result: anyhow::Result<()> = async move {
                OpenTimelineDatabase::setup_at_path(database_path).await?;
                Ok(())
            }
            .await;
            helper_print_result(result);
        }
        (Command::Backup, database_path, Some(data_dir)) => {
            let op = BackupMergeRestore::Backup;
            helper_database_operation(op, database_path.clone(), data_dir.clone()).await;
        }
        (Command::Merge, database_path, Some(data_dir)) => {
            let op = BackupMergeRestore::Merge;
            helper_database_operation(op, database_path.clone(), data_dir.clone()).await;
        }
        (Command::Restore, database_path, Some(data_dir)) => {
            let op = BackupMergeRestore::Restore;
            helper_database_operation(op, database_path.clone(), data_dir.clone()).await;
        }
        (Command::Stats, database_path, _) => {
            let result: anyhow::Result<DatabaseRowCount> = async move {
                let db_url = OpenTimelineDatabase::url_from_path(&database_path);
                let read_only = false;
                let pool = OpenTimelineDatabase::sqlite_pool_from_url(&db_url, read_only).await?;
                let mut transaction = pool.begin().await?;
                Ok(DatabaseRowCount::all(&mut transaction).await?)
            }
            .await;
            match result {
                Ok(count) => {
                    println!("Success");
                    println!(" - Entities          = {}", count.entities);
                    println!(" - Entity tags       = {}", count.entity_tags);
                    println!(" - Timelines         = {}", count.timelines);
                    println!(" - Subtimelines      = {}", count.subtimelines);
                    println!(" - Timeline entities = {}", count.timeline_entities);
                    println!(" - Timeline tags     = {}", count.timeline_tags);
                }
                Err(error) => {
                    eprintln!("Error: {error}");
                    std::process::exit(1);
                }
            };
        }
        //----------------------------------------------------------------------
        // Invalid
        //----------------------------------------------------------------------
        _ => {
            eprintln!("CLI Error: invalid options");
            Cli::command().print_long_help().unwrap();
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Database operation
enum BackupMergeRestore {
    Backup,
    Merge,
    Restore,
}

/// Helper to carry out database operation
async fn helper_database_operation(
    op: BackupMergeRestore,
    database_path: PathBuf,
    data_dir: PathBuf,
) {
    let result: anyhow::Result<()> = async move {
        // Connect to database
        let db_url = OpenTimelineDatabase::url_from_path(&database_path);
        let read_only = false;
        let pool = OpenTimelineDatabase::sqlite_pool_from_url(&db_url, read_only).await?;
        let mut transaction = pool.begin().await?;

        // Run operation
        match op {
            BackupMergeRestore::Backup => {
                OpenTimelineDatabase::backup_to_dir(&mut transaction, data_dir).await?
            }
            BackupMergeRestore::Merge => {
                OpenTimelineDatabase::merge_from_dir(&mut transaction, data_dir).await?
            }
            BackupMergeRestore::Restore => {
                OpenTimelineDatabase::restore_from_dir(&mut transaction, data_dir).await?
            }
        }

        // Commit changes
        transaction.commit().await?;
        Ok(())
    }
    .await;
    helper_print_result(result);
}

fn helper_print_result(result: anyhow::Result<()>) {
    match result {
        Ok(()) => println!("Success"),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    };
}

/// OpenTimeline CLI args using [clap]
#[derive(Parser, Debug)]
#[command(
    version,
    about = "OpenTimeline tool for basic database management",
    after_help = "This is intended for use when deploying to a server and in CI"
)]
pub struct Cli {
    // Database command
    #[arg(value_enum)]
    pub cli_command: Command,

    /// Path to the database
    #[arg(long)]
    pub database: PathBuf,

    /// Path to the dir that contains the JSON
    #[arg(long)]
    pub json: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum Command {
    Create,
    Backup,
    Restore,
    Merge,
    Stats,
}

impl ValueEnum for Command {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Create,
            Self::Backup,
            Self::Restore,
            Self::Merge,
            Self::Stats,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Command::Create => {
                Some(PossibleValue::new("create").help("Create a new database at the path"))
            }
            Command::Backup => Some(
                PossibleValue::new("backup")
                    .help("Backup the database at path to JSON in dir at path"),
            ),
            Command::Restore => Some(
                PossibleValue::new("restore")
                    .help("Restore the database at path from JSON in dir at path"),
            ),
            Command::Merge => Some(
                PossibleValue::new("merge")
                    .help("Merge into the database at path the JSON in dir at path"),
            ),
            Command::Stats => Some(PossibleValue::new("stats").help("Print database stats")),
        }
    }
}
