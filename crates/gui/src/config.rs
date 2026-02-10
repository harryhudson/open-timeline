// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! OpenTimeline GUI config
//!

use crate::app_colours::{AppColours, ColourTheme};
use directories_next::ProjectDirs;
use log::info;
use open_timeline_crud::{CrudError, setup_database_at_path};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

const PROJECT_QUALIFIER: &str = "org";
const ORG_NAME: &str = "OpenTimeline";
const APPLICATION_NAME: &str = "OpenTimeline";
const CONFIG_FILE_NAME: &str = "config.json";
const DEFAULT_DATABASE_FILE_NAME: &str = "timeline.sqlite";

pub type SharedConfig = Arc<RwLock<RuntimeConfig>>;

/// The config that's available across the application at runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub db_pool: SqlitePool,
    pub config: Config,
}

/// The config that's saved to disk
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Path to the database
    database_path: PathBuf,

    /// GUI colour theme
    pub colour_theme: ColourTheme,

    /// The custom theme
    pub custom_theme: AppColours,
}

impl Config {
    // TODO: this should assume that the config exists, because `ensure_exists`
    // exists and should have been called during start up.  We should assume
    // that the file it not directly touched
    pub fn load() -> Result<Self, CrudError> {
        info!("Loading config");
        let config_file_path = config_file_path()?;
        let data = fs::read_to_string(config_file_path)?;
        info!("JSON config loaded = {data}");
        let config: Config = serde_json::from_str(&data)?;
        info!("Config loaded = {config:?}");
        Ok(config)
    }

    pub fn set_to_default(&mut self) {
        let default = default_config();
        self.colour_theme = default.colour_theme();
        self.database_path = default.database_path();
    }

    pub fn colour_theme(&self) -> ColourTheme {
        self.colour_theme
    }

    pub fn set_colour_theme(&mut self, colour_theme: ColourTheme) {
        self.colour_theme = colour_theme.to_owned();
    }

    pub fn database_path(&self) -> PathBuf {
        self.database_path.clone()
    }

    pub fn set_database_path(&mut self, path: &PathBuf) {
        self.database_path = path.to_owned();
    }

    pub async fn ensure_setup() -> Result<(), CrudError> {
        info!("Ensuring config exists");
        let config_file_path = config_file_path()?;
        if !config_file_path.exists() {
            info!("No config file found");
            let new_config = default_config();
            new_config.save().await?;
            info!("Config created = {new_config:?}");
            setup_database_at_path(&new_config.database_path).await?;
            info!("Database setup at {}", &new_config.database_path.display());
        };
        info!("Config is setup");
        Ok(())
    }

    pub async fn save(&self) -> Result<(), CrudError> {
        // Setup database
        let path = self.database_path.to_owned();
        setup_database_at_path(&path).await?;

        // Save config to file
        let config_path = config_file_path()?;
        ensure_config_file_exists(&config_path)?;
        info!("Saving config to {config_path:?}");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, json)?;

        // Log success
        info!("Config saved");
        Ok(())
    }
}

/// Get the default config
fn default_config() -> Config {
    info!("Creating default config");
    let database_path = default_db_file_path();
    Config {
        colour_theme: ColourTheme::System,
        database_path,
        custom_theme: AppColours::default(),
    }
}

/// Get the project directories (e.g. where the config is stored)
#[cfg(debug_assertions)]
fn project_dirs() -> Result<ProjectDirs, CrudError> {
    info!("Getting project directories (dev build)");
    ProjectDirs::from(
        PROJECT_QUALIFIER,
        ORG_NAME,
        &format!("{APPLICATION_NAME} Dev"),
    )
    .ok_or(CrudError::Config)
}

/// Get the project directories (e.g. where the config is stored)
#[cfg(not(debug_assertions))]
fn project_dirs() -> Result<ProjectDirs, CrudError> {
    info!("Getting project directories");
    ProjectDirs::from(PROJECT_QUALIFIER, ORG_NAME, APPLICATION_NAME).ok_or(CrudError::Config)
}

/// Get the path to the config
fn config_file_path() -> Result<PathBuf, CrudError> {
    info!("Getting config file path");
    let config_file = project_dirs()?
        .config_dir()
        .to_path_buf()
        .join(CONFIG_FILE_NAME);
    info!("Config file path = {config_file:?}");
    Ok(config_file)
}

/// Ensure the config file exists (create if it doesn't)
fn ensure_config_file_exists(path: &PathBuf) -> Result<(), CrudError> {
    info!("Ensuring config path exists: {path:?}");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(path)?;
    Ok(())
}

/// Get the default path to the database
fn default_db_file_path() -> PathBuf {
    project_dirs()
        .unwrap()
        .data_dir()
        .to_path_buf()
        .join(DEFAULT_DATABASE_FILE_NAME)
}
