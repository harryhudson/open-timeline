// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Controls for backup/merge/restore to/from local files and web APIs
//!

use crate::config::SharedConfig;
use eframe::egui::{self, Align, Context, Grid, Layout, Response, Spinner, TextEdit, Ui};
use open_timeline_core::{Entity, TimelineEdit};
use open_timeline_crud::{BackupMergeRestore, BackupRestoreMergeError, backup, merge, restore};
use open_timeline_gui_core::{CheckForUpdates, Draw};
use open_timeline_gui_core::{DisplayStatus, GuiStatus};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;
use tempdir::TempDir;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::{Receiver, error::TryRecvError};

/// The backup|merge|restore GUI panel in the main window
#[derive(Debug)]
pub struct BackupMergeRestoreGui {
    /// Receive whether the operation suceceded or failed.
    rx_backup_restore_merge_update: Option<Receiver<Result<(), BackupRestoreMergeError>>>,

    /// Indicates which operation has been requested, if any.
    backup_merge_restore: Option<BackupMergeRestore>,

    /// The status of operations (which may be none)
    status: Status,

    /// Used to indirectly inform the rest of the application that a CRUD
    /// operation has been executed successfully (i.e. reloads may be required)
    tx_crud_operation_executed: UnboundedSender<()>,

    /// Database pool
    shared_config: SharedConfig,

    /// The OpenTimeline API endpoints
    open_timeline_api: ApiEndpoints,
}

/// Web API config for entities & timelines
#[derive(Debug)]
pub struct ApiEndpoints {
    entities: ApiEndpointConfig,
    timelines: ApiEndpointConfig,
}

/// A URL and whether it can be edited
#[derive(Debug)]
pub struct ApiEndpointConfig {
    url: String,
    enable_edit: bool,
}

/// The possible states of operation for the window
#[derive(Debug)]
enum Status {
    /// Nothing has been requested while the programme has ben running
    None,

    /// The operation last requested has succeeded
    Success(BackupMergeRestore),

    /// The operation last requested has failed
    Failure(BackupRestoreMergeError),

    /// The operation last requested is in progress
    InProgress,
}

impl DisplayStatus for Status {
    fn status_display(&self, ui: &mut Ui) -> Response {
        match &self {
            Self::None => ui.add(egui::Label::new(String::from("Ready")).truncate()),
            Self::Success(operation_requested) => {
                ui.add(egui::Label::new(format!("Success: {operation_requested:?}")).truncate())
            }
            Self::Failure(error) => ui.add(egui::Label::new(format!("Error: {error}")).truncate()),
            Self::InProgress => ui.add(Spinner::new()),
        }
    }
}

impl BackupMergeRestoreGui {
    /// Create a new backup|merge|restore GUI panel manager
    pub fn new(
        shared_config: SharedConfig,
        tx_crud_operation_executed: UnboundedSender<()>,
    ) -> Self {
        Self {
            rx_backup_restore_merge_update: None,
            backup_merge_restore: None,
            status: Status::None,
            tx_crud_operation_executed,
            shared_config,
            open_timeline_api: ApiEndpoints {
                entities: ApiEndpointConfig {
                    url: String::from("https://www.open-timeline.org/api/v1/entities/full"),
                    enable_edit: false,
                },
                timelines: ApiEndpointConfig {
                    url: String::from("https://www.open-timeline.org/api/v1/timelines/edit"),
                    enable_edit: false,
                },
            },
        }
    }

    /// Check for an update on the status of the operation requested
    fn check_for_msg(&mut self) {
        if let Some(backup_merge_restore) = &self.backup_merge_restore {
            if let Some(rx) = self.rx_backup_restore_merge_update.as_mut() {
                match rx.try_recv() {
                    Ok(result) => {
                        debug!("Recv backup|merge|restore update response");
                        match result {
                            Ok(()) => {
                                self.rx_backup_restore_merge_update = None;
                                self.status = Status::Success(backup_merge_restore.to_owned());
                                let _ = self.tx_crud_operation_executed.send(());
                            }
                            Err(error) => {
                                self.rx_backup_restore_merge_update = None;
                                self.status = Status::Failure(error);
                            }
                        }
                    }
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => (),
                }
            }
        }
    }

    /// Draw the current status
    fn draw_status(&mut self, ui: &mut Ui) {
        GuiStatus::display(ui, &self.status)
    }

    /// A helper to run the requested file operation.  This helps by providing a
    /// transaction to the target function, and commits it if the operation is
    /// successful.
    fn file_backup_restore_merge_helper(
        &mut self,
        target_dir: PathBuf,
        backup_merge_restore: BackupMergeRestore,
    ) {
        self.backup_merge_restore = Some(backup_merge_restore);
        self.status = Status::InProgress;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_backup_restore_merge_update = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        tokio::spawn(async move {
            let outer_result = async {
                let mut transaction = shared_config.read().await.db_pool.begin().await?;
                match backup_merge_restore {
                    BackupMergeRestore::Backup => backup(&mut transaction, target_dir).await?,
                    BackupMergeRestore::Merge => merge(&mut transaction, target_dir).await?,
                    BackupMergeRestore::Restore => restore(&mut transaction, target_dir).await?,
                }
                transaction
                    .commit()
                    .await
                    .map_err(BackupRestoreMergeError::Sqlx)?;
                Ok(())
            }
            .await;
            let _ = tx.send(outer_result).await;
        });
    }

    /// A helper to run the requested web API operation.  This helps by
    /// providing a transaction to the target function, and commits it if the
    /// operation is successful.
    fn web_api_restore_merge_helper(&mut self, backup_merge_restore: BackupMergeRestore) {
        self.backup_merge_restore = Some(backup_merge_restore);
        self.status = Status::InProgress;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_backup_restore_merge_update = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        let entities_url = self.open_timeline_api.entities.url.clone();
        let timelines_url = self.open_timeline_api.timelines.url.clone();
        debug!("entities_url = {entities_url}");
        debug!("timelines_url = {timelines_url}");
        tokio::spawn(async move {
            let outer_result: Result<(), BackupRestoreMergeError> = async {
                // Fetch
                let (timelines, entities) = {
                    // TODO: known bug: reduced timelines are accepted (results in a success,
                    // but really the timelines are all empty)
                    // Fetch the timelines (smaller response first)
                    let response_timelines =
                        reqwest::get(timelines_url).await?.error_for_status()?;
                    let timelines: Vec<TimelineEdit> = response_timelines.json().await?;

                    // Fetch the entities
                    let response_entities = reqwest::get(entities_url).await?.error_for_status()?;
                    let entities: Vec<Entity> = response_entities.json().await?;

                    debug!(
                        "Fetched timelines from web API (count = {})",
                        timelines.len()
                    );
                    debug!("Fetched entities from web API (count = {})", entities.len());
                    (timelines, entities)
                };

                // Save fetched to tmp file
                let dir = {
                    // Save the entities & timelines to files
                    let tmp_dir = TempDir::new("open-timeline-gui-web-restore-merge")?.into_path();
                    debug!("tmp dir = {}", tmp_dir.display());

                    // Timelines
                    let timelines_file = File::create(tmp_dir.join("timelines.json"))?;
                    let timelines_writer = BufWriter::new(timelines_file);
                    serde_json::to_writer_pretty(timelines_writer, &timelines)?;
                    debug!("Wrote timelines to tmp file");

                    // Entities
                    let entities_file = File::create(tmp_dir.join("entities.json"))?;
                    let entities_writer = BufWriter::new(entities_file);
                    serde_json::to_writer_pretty(entities_writer, &entities)?;
                    debug!("Wrote entities to tmp file");

                    // Return the dir if all ok
                    tmp_dir
                };

                // Merge or restore
                let mut transaction = shared_config.read().await.db_pool.begin().await?;
                match backup_merge_restore {
                    BackupMergeRestore::Backup => (),
                    BackupMergeRestore::Merge => merge(&mut transaction, dir).await?,
                    BackupMergeRestore::Restore => restore(&mut transaction, dir).await?,
                }
                transaction
                    .commit()
                    .await
                    .map_err(BackupRestoreMergeError::Sqlx)?;
                Ok(())
            }
            .await;
            let _ = tx.send(outer_result).await;
        });
    }

    /// Draw controls for backup/merge/restore to/from local files
    fn draw_file_backup_merge_restore(&mut self, ui: &mut Ui) {
        open_timeline_gui_core::Label::sub_heading(ui, "File");
        let description =
            "Backup, merge, and restore to & from JSON files containing entities & timelines";
        open_timeline_gui_core::Label::description(ui, description);
        ui.add_space(5.0);

        let width = ui.available_width() / 3.0;
        Grid::new("file_buttons")
            .min_col_width(width)
            .max_col_width(width)
            .num_columns(3)
            .show(ui, |ui| {
                // "Backup" button
                if open_timeline_gui_core::Button::tall_full_width(ui, "Backup").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.file_backup_restore_merge_helper(path, BackupMergeRestore::Backup);
                    }
                }

                // "Merge In" button
                if open_timeline_gui_core::Button::tall_full_width(ui, "Merge In").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.file_backup_restore_merge_helper(path, BackupMergeRestore::Merge);
                    }
                }

                // "Restore" button
                if open_timeline_gui_core::Button::tall_full_width(ui, "Restore").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.file_backup_restore_merge_helper(path, BackupMergeRestore::Restore);
                    }
                }
            });
    }

    /// Draw controls for merge/restore from JSON web API
    fn draw_web_api_merge_restore(&mut self, ui: &mut Ui) {
        // Heading
        open_timeline_gui_core::Label::sub_heading(ui, "Web API");

        // Description
        let description = "Merge in or restore from a JSON web API serving entities & timelines.  The default URLs are for the OpenTimeline web JSON API";
        open_timeline_gui_core::Label::description(ui, description);
        ui.add_space(5.0);

        // API endpoints
        Grid::new("URLs").num_columns(2).show(ui, |ui| {
            // Entities
            draw_api_endpoint_config(ui, "Entities URL", &mut self.open_timeline_api.entities);
            ui.end_row();

            // Timelines
            draw_api_endpoint_config(ui, "Timelines URL", &mut self.open_timeline_api.timelines);
            ui.end_row();
        });
        ui.add_space(5.0);

        // Backup/merge/restore buttons
        let width = ui.available_width() / 3.0;
        Grid::new("url_buttons")
            .min_col_width(width)
            .max_col_width(width)
            .num_columns(3)
            .show(ui, |ui| {
                // "Backup" button
                ui.add_enabled_ui(false, |ui| {
                    open_timeline_gui_core::Button::tall_full_width(ui, "Backup");
                });

                // "Merge In" button
                if open_timeline_gui_core::Button::tall_full_width(ui, "Merge In").clicked() {
                    self.web_api_restore_merge_helper(BackupMergeRestore::Merge);
                }

                // "Restore" button
                if open_timeline_gui_core::Button::tall_full_width(ui, "Restore").clicked() {
                    self.web_api_restore_merge_helper(BackupMergeRestore::Restore);
                }
            });
    }
}

impl Draw for BackupMergeRestoreGui {
    fn draw(&mut self, _ctx: &Context, ui: &mut Ui) {
        // Status
        self.draw_status(ui);
        ui.separator();

        // Description
        let description =
            "This panel facilitates backing up, restoring, and merging in entities & timelines";
        open_timeline_gui_core::Label::description(ui, description);
        ui.separator();

        // File
        self.draw_file_backup_merge_restore(ui);
        ui.add_space(15.0);

        // Web API
        self.draw_web_api_merge_restore(ui);
    }
}

impl CheckForUpdates for BackupMergeRestoreGui {
    fn check_for_updates(&mut self) {
        self.check_for_msg();
    }

    fn waiting_for_updates(&mut self) -> bool {
        let waiting = self.rx_backup_restore_merge_update.is_some();
        if waiting {
            info!("BackupMergeRestoreGui is waiting for updates");
        }
        waiting
    }
}

/// Draw an API endpoint config
fn draw_api_endpoint_config(ui: &mut Ui, label: &str, api_endpoint: &mut ApiEndpointConfig) {
    open_timeline_gui_core::Label::strong(ui, label);
    ui.allocate_ui_with_layout(
        ui.available_size(),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.checkbox(&mut api_endpoint.enable_edit, "Enable editing");
            let input = TextEdit::singleline(&mut api_endpoint.url).desired_width(f32::INFINITY);
            ui.add_enabled(api_endpoint.enable_edit, input);
        },
    );
}
