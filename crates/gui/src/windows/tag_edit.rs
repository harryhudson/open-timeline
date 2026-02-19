// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The GUI window for bulk editing tag
//!

// TODO: bulk edit timeline vs entity vs both tags

use crate::app::ActionRequest;
use crate::common::*;
use crate::components::TagGui;
use crate::config::SharedConfig;
use crate::consts::DEFAULT_WINDOW_SIZES;
use crate::shortcuts::global_shortcuts;
use crate::windows::{Deleted, DeletedStatus};
use bool_tag_expr::Tag;
use eframe::egui::{CentralPanel, Context, Vec2, ViewportId};
use open_timeline_crud::{CrudError, delete_all_matching_tags, update_all_matching_entity_tags};
use open_timeline_gui_core::{
    BreakOutWindow, CheckForUpdates, Draw, Reload, Valid, ValidityAsynchronous, window_has_focus,
};
use open_timeline_gui_core::{Shortcut, ShowRemoveButton};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::{Receiver, error::TryRecvError};

/// Edit a tag
#[derive(Debug)]
pub struct TagBulkEditGui {
    /// The current [`Tag`]
    database_entry: Tag,

    /// The GUI component for inputting/validating/etc the new tag
    new_tag_gui: TagGui,

    /// Whether the tag has been completely deleted or not.  If it has been, the
    /// `Deleted` variant holds the `Instant` it was deleted.
    deleted_status: DeletedStatus,

    /// The status printed for the user
    status_str: String,

    /// Receive update operation updates (if an update has been requested)
    rx_update: Option<Receiver<Result<(), CrudError>>>,

    /// Receive delete operation updates (if a deletion has been requested)
    rx_delete: Option<Receiver<Result<(), CrudError>>>,

    /// Send an action request to the main loop
    tx_action_request: UnboundedSender<ActionRequest>,

    /// Used to indirectly inform the rest of the application that a CRUD
    /// operation has been executed
    tx_crud_operation_executed: UnboundedSender<()>,

    /// Whether this window should be closed or not
    wants_to_be_closed: bool,

    /// Database pool
    shared_config: SharedConfig,
}

impl TagBulkEditGui {
    /// Create new `TagBulkEditGui`
    pub fn new(
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
        tx_crud_operation_executed: UnboundedSender<()>,
        tag: Tag,
    ) -> Self {
        TagBulkEditGui {
            database_entry: tag.clone(),
            new_tag_gui: TagGui::from_tag(tag, ShowRemoveButton::No),
            deleted_status: DeletedStatus::NotDeleted,
            status_str: String::from("No changes to save"),
            rx_update: None,
            rx_delete: None,
            tx_action_request,
            tx_crud_operation_executed,
            wants_to_be_closed: false,
            shared_config,
        }
    }

    /// Get the tag being edited
    pub fn tag(&self) -> &Tag {
        &self.database_entry
    }

    fn request_update(&mut self) {
        if self.has_been_deleted() {
            return;
        }
        let validity = self.new_tag_gui.validity();
        match validity {
            ValidityAsynchronous::Valid => {
                let as_opentimeline_type = self.new_tag_gui.to_opentimeline_type();
                self.update(as_opentimeline_type);
            }
            ValidityAsynchronous::Invalid(error) => {
                self.status_str = format!("Tag can't be updated (error: {error})");
            }
            ValidityAsynchronous::Waiting => {
                self.status_str = String::from("Waiting for validation")
            }
        }
    }

    fn request_delete(&mut self) {
        let validity = self.new_tag_gui.validity();
        match validity {
            ValidityAsynchronous::Valid => {
                let as_opentimeline_type = self.new_tag_gui.to_opentimeline_type();
                self.delete(as_opentimeline_type);
            }
            ValidityAsynchronous::Invalid(error) => {
                self.status_str = format!("Tag can't be deleted (error: {error})");
            }
            ValidityAsynchronous::Waiting => {
                self.status_str = String::from("Waiting for validation")
            }
        }
    }

    fn update(&mut self, new_tag: Tag) {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_update = Some(rx);
        let old_tag = self.tag().to_owned();
        let shared_config = Arc::clone(&self.shared_config);
        tokio::spawn(async move {
            let result = async {
                let mut transaction = shared_config.read().await.db_pool.begin().await?;
                let _ = update_all_matching_entity_tags(&mut transaction, old_tag, new_tag).await?;
                // TODO: is this the correct error variant?
                transaction.commit().await.map_err(|_| CrudError::DbError)?;
                Ok(())
            }
            .await;
            let _ = tx.send(result).await;
        });
    }

    fn delete(&mut self, tag: Tag) {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_delete = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        tokio::spawn(async move {
            let result = async {
                let mut transaction = shared_config.read().await.db_pool.begin().await?;
                delete_all_matching_tags(&mut transaction, tag).await?;
                // TODO: is this the correct error variant?
                transaction.commit().await.map_err(|_| CrudError::DbError)?;
                Ok(())
            }
            .await;
            let _ = tx.send(result).await;
        });
    }

    // TODO: Nearly identical to that in entity.rs (make generic or macro)
    /// Handle create/update/delete response
    fn check_for_crud_status_updates(&mut self) {
        // Response to create/update request
        if let Some(rx) = self.rx_update.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    self.rx_update = None;
                    match result {
                        Ok(()) => {
                            self.status_str = String::from("Updated tag");
                            // TODO: this could fail (become invalid) - send back the new tag from database
                            self.database_entry = self.new_tag_gui.to_opentimeline_type();
                            let _ = self.tx_crud_operation_executed.send(());
                        }
                        Err(error) => {
                            self.status_str = format!("Failed to update tag: {error}");
                        }
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }

        // Response to delete request
        if let Some(rx) = self.rx_delete.as_mut() {
            let deleted_tag = self.database_entry.clone();
            match rx.try_recv() {
                Ok(result) => {
                    self.rx_delete = None;
                    match result {
                        Ok(()) => {
                            self.status_str = format!("Sucessfully deleted '{deleted_tag}'");
                            self.set_deleted_status(DeletedStatus::Deleted(Instant::now()));
                            let _ = self.tx_crud_operation_executed.send(());
                        }
                        Err(error) => {
                            self.status_str = format!("Failed to delete '{deleted_tag}': {error}")
                        }
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }

    // TODO: trait
    /// Set the current tag from the tag passed in
    fn reset(&mut self) {
        self.new_tag_gui = TagGui::from_tag(self.database_entry.clone(), ShowRemoveButton::No)
    }

    // TODO: use an enum instead of Option<bool>
    // TODO: trait
    /// Whether the tag differs from the one in the database
    fn differs_from_database_entry(&self) -> Option<bool> {
        if self.new_tag_gui.validity() == ValidityAsynchronous::Valid {
            let current_tag = self.new_tag_gui.to_opentimeline_type();
            Some(current_tag != self.database_entry)
        } else {
            None
        }
    }
}

impl Reload for TagBulkEditGui {
    fn request_reload(&mut self) {
        // TODO: if the Tag is no longer in the database (deleted elsewhere),
        // close the window
    }

    fn check_reload_response(&mut self) {
        // TODO: see request_reload()
    }
}

impl Deleted for TagBulkEditGui {
    fn set_deleted_status(&mut self, deleted_status: DeletedStatus) {
        self.deleted_status = deleted_status;
    }

    fn deleted_status(&self) -> DeletedStatus {
        self.deleted_status
    }
}

impl CheckForUpdates for TagBulkEditGui {
    fn check_for_updates(&mut self) {
        self.check_reload_response();
        self.check_for_crud_status_updates();
    }

    fn waiting_for_updates(&mut self) -> bool {
        let waiting = self.rx_update.is_some() || self.rx_delete.is_some();
        if waiting {
            info!("TagBulkEditGui is waiting for updates");
        }
        waiting
    }
}

impl BreakOutWindow for TagBulkEditGui {
    fn draw(&mut self, ctx: &Context) {
        // Handle shortcuts
        if window_has_focus(ctx) {
            if Shortcut::save(ctx) {
                self.request_update();
            }
            if Shortcut::close_window(ctx) {
                self.wants_to_be_closed = true;
            }
        }

        // Check for global shortcuts
        global_shortcuts(ctx, &mut self.tx_action_request);

        CentralPanel::default().show(ctx, |ui| {
            // Window title
            open_timeline_gui_core::Label::heading(ui, "Tag");
            ui.separator();

            // Status
            ui.label(&self.status_str);
            ui.separator();

            // Display emtpy window and countdown after deletion
            if self.has_been_deleted() {
                self.draw_deleted_message(ctx, ui);

                //
                if let DeletedStatus::Deleted(deleted_at) = self.deleted_status() {
                    let elapsed_secs = deleted_at.elapsed().as_secs() as i32;
                    let remaining_seconds = 5 - elapsed_secs;
                    if remaining_seconds < 1 {
                        self.wants_to_be_closed = true;
                    }
                }

                return;
            }

            // Create/Update/Delete buttons
            ui.horizontal(|ui| {
                // Delete comes first so that it never moves (reduced likelihood
                // of accidentally clicking it)
                if open_timeline_gui_core::Button::delete(ui).clicked() {
                    self.request_delete();
                }
                // Can be invalid or valid, but cannot be equal to the entry in the database
                if self.differs_from_database_entry() != Some(false)
                    && open_timeline_gui_core::Button::reset(ui).clicked()
                {
                    self.reset();
                }
                // Must be valid & differ from the database
                if self.differs_from_database_entry() == Some(true)
                    && self.new_tag_gui.validity() == ValidityAsynchronous::Valid
                    && open_timeline_gui_core::Button::update(ui).clicked()
                {
                    self.request_update();
                }
            });
            ui.separator();

            // Existing tag
            open_timeline_gui_core::Label::sub_heading(ui, "Existing");
            ui.label(format!("{}", self.database_entry));
            ui.separator();

            // New tag
            open_timeline_gui_core::Label::sub_heading(ui, "New");
            ui.add_enabled_ui(true, |ui| self.new_tag_gui.draw(ctx, ui));
        });
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(
            DEFAULT_WINDOW_SIZES.tag_edit.width,
            DEFAULT_WINDOW_SIZES.tag_edit.height,
        )
    }

    fn viewport_id(&mut self) -> ViewportId {
        ViewportId(eframe::egui::Id::from({
            let mut hasher = DefaultHasher::new();
            self.tag().hash(&mut hasher);
            format!("tag_edit_{}", hasher.finish())
        }))
    }

    fn title(&mut self) -> String {
        format!("Edit Tag • {}", self.tag())
    }

    fn wants_to_be_closed(&mut self) -> bool {
        self.wants_to_be_closed
    }
}
