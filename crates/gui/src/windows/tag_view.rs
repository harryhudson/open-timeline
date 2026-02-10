// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The GUI for viewing all entities and timelines that have a particular tag
//!

use crate::app::ActionRequest;
use crate::config::SharedConfig;
use crate::consts::DEFAULT_WINDOW_SIZES;
use crate::shortcuts::global_shortcuts;
use crate::spawn_transaction_no_commit_send_result;
use crate::windows::{Deleted, DeletedStatus};
use bool_tag_expr::Tag;
use eframe::egui::{CentralPanel, Context, ScrollArea, Vec2, ViewportId};
use open_timeline_core::{IsReducedCollection, IsReducedType};
use open_timeline_crud::{CrudError, FetchAllWithTag, ReducedAll};
use open_timeline_gui_core::{BreakOutWindow, Reload, Shortcut, window_has_focus};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, UnboundedSender};

/// Edit a tag
#[derive(Debug)]
pub struct TagViewGui {
    /// The tag currently being viewing
    tag: Tag,

    /// The status string
    status_str: String,

    /// Send an action request to the main loop
    tx_action_request: UnboundedSender<ActionRequest>,

    /// All the entities and timelines that have the tag
    all_with_tag: Option<ReducedAll>,

    /// Receive all entities & timelines that have the tag
    rx_reload: Option<Receiver<Result<ReducedAll, CrudError>>>,

    /// Whether or not a reload has been requested
    requested_reload: bool,

    /// Whether the tag has been completely deleted or not.  If it has been, the
    /// `Deleted` variant holds the `Instant` this window became aware of the
    /// fact.
    deleted_status: DeletedStatus,

    /// Whether this window should be closed or not
    wants_to_be_closed: bool,

    /// Database pool
    shared_config: SharedConfig,
}

impl TagViewGui {
    /// Create new TagViewGui
    pub fn new(
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
        tag: Tag,
    ) -> Self {
        let mut tag_view_gui = TagViewGui {
            tag,
            status_str: String::from("View tag"),
            tx_action_request,
            all_with_tag: None,
            rx_reload: None,
            requested_reload: false,
            deleted_status: DeletedStatus::NotDeleted,
            wants_to_be_closed: false,
            shared_config,
        };
        tag_view_gui.request_reload();
        tag_view_gui
    }

    /// Get the tag being viewed.
    pub fn tag(&self) -> &Tag {
        &self.tag
    }
}

impl Reload for TagViewGui {
    fn request_reload(&mut self) {
        if self.has_been_deleted() {
            return;
        }
        self.requested_reload = true;
        let tag = self.tag.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_reload = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        spawn_transaction_no_commit_send_result!(
            shared_config,
            bounded,
            tx,
            |transaction| async move { ReducedAll::fetch_all_with_tag(transaction, &tag).await }
        );
    }

    fn check_reload_response(&mut self) {
        if let Some(rx) = self.rx_reload.as_mut() {
            match rx.try_recv() {
                Ok(received) => {
                    self.rx_reload = None;
                    self.requested_reload = false;
                    match received {
                        Ok(all) => {
                            if all.entities().collection().is_empty()
                                && all.timelines().collection().is_empty()
                            {
                                self.set_deleted_status(DeletedStatus::Deleted(Instant::now()));
                                return;
                            }
                            self.status_str = String::from("Sucessfully fetched");
                            self.all_with_tag = Some(all);
                        }
                        // TODO: deleted?
                        Err(error) => {
                            eprintln!("Tag view error: {error}");
                            self.status_str = format!("Error fetching: {error}");
                        }
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }
}

impl Deleted for TagViewGui {
    fn set_deleted_status(&mut self, deleted_status: DeletedStatus) {
        self.deleted_status = deleted_status;
    }

    fn deleted_status(&self) -> DeletedStatus {
        self.deleted_status
    }
}

impl BreakOutWindow for TagViewGui {
    fn draw(&mut self, ctx: &Context) {
        // Handle shortcuts
        if window_has_focus(ctx) && Shortcut::close_window(ctx) {
            self.wants_to_be_closed = true;
        }

        // Check for global shortcuts
        global_shortcuts(ctx, &mut self.tx_action_request);

        // Check for reload
        self.check_reload_response();

        CentralPanel::default().show(ctx, |ui| {
            // Tag
            open_timeline_gui_core::Label::heading(ui, &format!("{}", self.tag));
            ui.label("Tag");
            ui.separator();

            // Has been deleted
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

            // Status
            ui.label(&self.status_str);
            ui.separator();

            let available_width = ui.available_width();
            let available_height = ui.available_height();
            let scroll_height = ((available_height - (2.0 * 20.0)) / 2.0).max(0.0);

            // Entities
            open_timeline_gui_core::Label::sub_heading(ui, "Entities");
            ScrollArea::vertical()
                .max_height(scroll_height)
                .id_salt(format!("{:?}_entities_scroll_area", self.tag))
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::from([available_width, scroll_height]));
                    match &self.all_with_tag {
                        Some(all_reduced) => {
                            let entities = all_reduced.entities();
                            if entities.collection().is_empty() {
                                open_timeline_gui_core::Label::none(ui);
                            } else {
                                for entity in all_reduced.entities() {
                                    ui.label(entity.name().as_str());
                                }
                            }
                        }
                        None => {
                            open_timeline_gui_core::Label::none(ui);
                        }
                    }
                });
            ui.separator();

            // Timelines
            open_timeline_gui_core::Label::sub_heading(ui, "Timelines");
            ScrollArea::vertical()
                .max_height(ui.available_height())
                .id_salt(format!("{:?}_timelines_scroll_area", self.tag))
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::from([available_width, ui.available_height()]));
                    match &self.all_with_tag {
                        Some(all_reduced) => {
                            let timelines = all_reduced.timelines();
                            if timelines.collection().is_empty() {
                                open_timeline_gui_core::Label::none(ui);
                            } else {
                                for timeline in all_reduced.timelines() {
                                    ui.label(timeline.name().as_str());
                                }
                            }
                        }
                        None => {
                            open_timeline_gui_core::Label::none(ui);
                        }
                    }
                });
        });
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(
            DEFAULT_WINDOW_SIZES.tag_view.width,
            DEFAULT_WINDOW_SIZES.tag_view.height,
        )
    }

    fn viewport_id(&mut self) -> ViewportId {
        ViewportId(eframe::egui::Id::from({
            let mut hasher = DefaultHasher::new();
            self.tag().hash(&mut hasher);
            format!("tag_view_{}", hasher.finish())
        }))
    }

    fn title(&mut self) -> String {
        format!("View Tag • {}", self.tag)
    }

    fn wants_to_be_closed(&mut self) -> bool {
        self.wants_to_be_closed
    }
}
