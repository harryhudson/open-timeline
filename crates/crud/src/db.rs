// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Create & migrate SQLite database files for OpenTimeline
//!

use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};
use std::path::Path;

/// Setup a database at the supplied path (ensure the file exists and run the
/// migrations
pub async fn setup_database_at_path(path: &Path) -> Result<(), sqlx::Error> {
    // Construct the database URL
    let db_url = db_url_from_path(path);

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create the database file (if not already extant)
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        println!("Creating database at {}", path.to_string_lossy());
        Sqlite::create_database(&db_url).await?;
    } else {
        println!("Database already exists at {}", path.to_string_lossy());
    }

    // Open a connection
    let pool = SqlitePool::connect(&db_url).await?;

    // Run migrations (uses compile-time embedding of migrations)
    sqlx::migrate!("./migrations").run(&pool).await?;

    println!(
        "Migrations applied successfully to {}",
        path.to_string_lossy()
    );

    Ok(())
}

/// Create a URL for the SQLite database using the path to the database
pub fn db_url_from_path(path: &Path) -> String {
    format!("sqlite://{}", path.to_string_lossy())
}
