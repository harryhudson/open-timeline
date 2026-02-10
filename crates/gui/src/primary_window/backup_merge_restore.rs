// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Desktop GUI stats
//!

use crate::config::SharedConfig;
use eframe::egui::{self, Context, Response, Spinner, Ui};
use open_timeline_crud::{BackupMergeRestore, BackupRestoreMergeError, backup, merge, restore};
use open_timeline_gui_core::Draw;
use open_timeline_gui_core::{DisplayStatus, GuiStatus};
use std::path::PathBuf;
use std::sync::Arc;
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
        }
    }

    /// Check for an update on the status of the operation requested
    fn check_for_msg(&mut self) {
        if let Some(backup_merge_restore) = &self.backup_merge_restore {
            if let Some(rx) = self.rx_backup_restore_merge_update.as_mut() {
                match rx.try_recv() {
                    Ok(result) => match result {
                        // The operation succeeded
                        Ok(()) => {
                            self.rx_backup_restore_merge_update = None;
                            self.status = Status::Success(backup_merge_restore.to_owned());
                            let _ = self.tx_crud_operation_executed.send(());
                        }

                        // The operation failed
                        Err(error) => {
                            self.rx_backup_restore_merge_update = None;
                            self.status = Status::Failure(error);
                        }
                    },
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

    /// A helper to run the requested operation.  This helps by providing a
    /// transaction to the target function, and commits it if the operation is
    /// successful.
    fn backup_restore_merge_helper(
        &mut self,
        path: PathBuf,
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
                    BackupMergeRestore::Backup => backup(&mut transaction, path).await?,
                    BackupMergeRestore::Merge => merge(&mut transaction, path).await?,
                    BackupMergeRestore::Restore => restore(&mut transaction, path).await?,
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
}

impl Draw for BackupMergeRestoreGui {
    fn draw(&mut self, _ctx: &Context, ui: &mut Ui) {
        self.check_for_msg();

        self.draw_status(ui);
        ui.separator();

        open_timeline_gui_core::Label::sub_heading(ui, "JSON");
        ui.separator();

        // "Backup" button
        if open_timeline_gui_core::Button::tall_full_width(ui, "Backup").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.backup_restore_merge_helper(path, BackupMergeRestore::Backup);
            }
        }

        // "Merge In" button
        if open_timeline_gui_core::Button::tall_full_width(ui, "Merge In").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.backup_restore_merge_helper(path, BackupMergeRestore::Merge);
            }
        }

        // "Restore" button
        if open_timeline_gui_core::Button::tall_full_width(ui, "Restore").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.backup_restore_merge_helper(path, BackupMergeRestore::Restore);
            }
        }
    }
}
