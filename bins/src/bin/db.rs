// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! *Part of the wider OpenTimeline project*
//!
//! The OpenTimeline website ([www.open-timeline.org](www.open-timeline.org))
//!

use clap::{CommandFactory, Parser, ValueEnum, builder::PossibleValue};
use open_timeline_crud::{db_url_from_path, restore, setup_database_at_path};
use sqlx::{Connection, SqliteConnection};
use std::path::PathBuf;

/// OpenTimeline entry point
///
/// One of:
/// - Backup the database
/// - Restore the database
/// - Serve the website and API
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Check the options
    match (&args.cli_command, &args.database, &args.json) {
        //----------------------------------------------------------------------
        // Valid
        //----------------------------------------------------------------------
        (Command::Create, database, _) => match setup_database_at_path(database).await {
            Ok(()) => println!("Success"),
            Err(error) => {
                eprintln!("Error: {error}");
                std::process::exit(1);
            }
        },
        (Command::Backup, _database, Some(_json)) => {
            todo!()
        }
        (Command::Restore, database, Some(json)) => {
            // Generate database URL
            let db_url = db_url_from_path(database);

            // Open database connection
            let mut connection = match SqliteConnection::connect(&db_url).await {
                Ok(connection) => connection,
                Err(error) => {
                    eprintln!("Error connecting to database: {error}");
                    std::process::exit(1);
                }
            };

            // Begin database transaction
            let mut transaction: sqlx::Transaction<'_, sqlx::Sqlite> =
                match connection.begin().await {
                    Ok(transaction) => transaction,
                    Err(error) => {
                        eprintln!("Error starting transaction: {error}");
                        std::process::exit(1);
                    }
                };

            // Restore the database
            match restore(&mut transaction, json.to_owned()).await {
                Ok(()) => (),
                Err(error) => {
                    eprintln!("Error restoring database: {error}");
                    std::process::exit(1);
                }
            }

            // Commit the transaction
            match transaction.commit().await {
                Ok(()) => println!("Sucessfully restored database"),
                Err(error) => {
                    eprintln!("Error committing transaction: {error}");
                    std::process::exit(1);
                }
            }
        }
        (Command::Merge, _database, Some(_json)) => {
            todo!()
        }
        (Command::Stats, _database, _) => {
            todo!()
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
