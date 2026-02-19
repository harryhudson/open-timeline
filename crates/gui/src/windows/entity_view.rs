// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The view entity GUI
//!

use crate::app::ActionRequest;
use crate::config::SharedConfig;
use crate::consts::DEFAULT_WINDOW_SIZES;
use crate::shortcuts::global_shortcuts;
use crate::{
    spawn_transaction_no_commit_send_result,
    windows::{Deleted, DeletedStatus},
};
use eframe::egui::{
    self, Align, CentralPanel, Context, Layout, RichText, ScrollArea, Ui, Vec2, ViewportId,
};
use egui_extras::{Column, TableBuilder};
use open_timeline_core::{Entity, HasIdAndName, OpenTimelineId};
use open_timeline_crud::{CrudError, FetchById};
use open_timeline_gui_core::{
    BreakOutWindow, CheckForUpdates, Reload, body_text_height, widget_x_spacing,
};
use open_timeline_gui_core::{Shortcut, window_has_focus};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, UnboundedSender};

/// View an entity
#[derive(Debug)]
pub struct EntityViewGui {
    /// The ID of the entity being viewed
    entity_id: OpenTimelineId,

    /// The entity being viewed.  This is `None` until it has been fetched.
    entity: Option<Entity>,

    /// Receive reloaded data
    rx_reload: Option<Receiver<Result<Entity, CrudError>>>,

    /// Send an action request to the main loop
    tx_action_request: UnboundedSender<ActionRequest>,

    /// Whether or not a reload has been requested
    requested_reload: bool,

    /// Whether the entity has been deleted or not.  If it has been, the
    /// `Deleted` variant holds the `Instant` this window became aware of the
    /// fact.
    deleted_status: DeletedStatus,

    /// Whether this window should be closed or not
    wants_to_be_closed: bool,

    /// Database pool
    shared_config: SharedConfig,
}

impl EntityViewGui {
    /// Create new EntityViewGui
    pub fn new(
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
        entity_id: OpenTimelineId,
    ) -> Self {
        let mut entity_view_gui = EntityViewGui {
            entity_id,
            entity: None,
            rx_reload: None,
            tx_action_request,
            requested_reload: false,
            deleted_status: DeletedStatus::NotDeleted,
            wants_to_be_closed: false,
            shared_config,
        };
        entity_view_gui.request_reload();
        entity_view_gui
    }

    /// Get the ID of the entity being viewed
    pub fn entity_id(&self) -> OpenTimelineId {
        self.entity_id
    }
}

impl Reload for EntityViewGui {
    fn request_reload(&mut self) {
        if self.has_been_deleted() {
            return;
        }
        self.requested_reload = true;
        let entity_id = self.entity_id;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_reload = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        spawn_transaction_no_commit_send_result!(
            shared_config,
            bounded,
            tx,
            |transaction| async move { Entity::fetch_by_id(transaction, &entity_id).await }
        );
    }

    fn check_reload_response(&mut self) {
        if let Some(rx) = self.rx_reload.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    self.rx_reload = None;
                    self.requested_reload = false;
                    match result {
                        Ok(entity) => self.entity = Some(entity),
                        Err(CrudError::IdNotInDb) => {
                            self.set_deleted_status(DeletedStatus::Deleted(Instant::now()))
                        }
                        Err(_) => todo!(),
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }
}

impl Deleted for EntityViewGui {
    fn set_deleted_status(&mut self, deleted_status: DeletedStatus) {
        self.deleted_status = deleted_status;
    }

    fn deleted_status(&self) -> DeletedStatus {
        self.deleted_status
    }
}

impl CheckForUpdates for EntityViewGui {
    fn check_for_updates(&mut self) {
        self.check_reload_response();
    }

    fn waiting_for_updates(&mut self) -> bool {
        let waiting = self.rx_reload.is_some();
        if waiting {
            info!("EntityViewGui is waiting for updates");
        }
        waiting
    }
}

impl BreakOutWindow for EntityViewGui {
    fn draw(&mut self, ctx: &Context) {
        // Handle shortcuts
        if window_has_focus(ctx) && Shortcut::close_window(ctx) {
            self.wants_to_be_closed = true;
        }

        // Check for global shortcuts
        global_shortcuts(ctx, &mut self.tx_action_request);

        CentralPanel::default().show(ctx, |ui| {
            if self.requested_reload {
                ui.spinner();
                return;
            }

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

            let entity = self.entity.as_mut().unwrap();
            let available_width = ui.available_width();
            let row_height = body_text_height(ui);
            let spacing = widget_x_spacing(ui);
            let column_width = (available_width - spacing) / 2.0;

            // Name
            open_timeline_gui_core::Label::heading(ui, entity.name().as_str());
            ui.label(RichText::new("Entity").weak());
            ui.separator();

            // Dates
            let start_date_str = entity.start().as_long_date_format();
            let end_date_str = entity
                .end()
                .map(|date| date.as_long_date_format())
                .unwrap_or_default();
            let label_height = body_text_height(ui);
            ui.add_sized(
                [available_width, label_height],
                egui::Label::new(format!("{start_date_str}   –   {end_date_str}")),
            );
            ui.separator();

            // Tags
            open_timeline_gui_core::Label::sub_heading(ui, "Tags");
            if let Some(tags) = entity.tags() {
                ScrollArea::vertical().show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .column(Column::exact(column_width))
                        .column(Column::exact(column_width))
                        .body(|mut body| {
                            for tag in tags {
                                body.row(row_height, |mut row| {
                                    // Tag name
                                    row.col(|ui| {
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                let name = match &tag.name {
                                                    Some(name) => name.as_str(),
                                                    None => "",
                                                };
                                                ui.add(egui::Label::new(name).truncate());
                                            },
                                        );
                                    });
                                    // Tag value
                                    row.col(|ui: &mut Ui| {
                                        ui.with_layout(
                                            Layout::left_to_right(Align::Center),
                                            |ui| {
                                                ui.add(
                                                    egui::Label::new(tag.value.as_str()).truncate(),
                                                );
                                            },
                                        );
                                    });
                                });
                            }
                        });
                });
            } else {
                open_timeline_gui_core::Label::none(ui);
            }
        });
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(
            DEFAULT_WINDOW_SIZES.entity_view.width,
            DEFAULT_WINDOW_SIZES.entity_view.height,
        )
    }

    fn viewport_id(&mut self) -> ViewportId {
        ViewportId(eframe::egui::Id::from(format!(
            "entity_view_{}",
            self.entity_id()
        )))
    }

    fn title(&mut self) -> String {
        match self.entity.as_ref() {
            None => String::from("View Entity  -  [loading]"),
            Some(entity) => format!("View Entity • {}", entity.name().as_str()),
        }
    }

    fn wants_to_be_closed(&mut self) -> bool {
        self.wants_to_be_closed
    }
}
