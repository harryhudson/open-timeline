// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The view timeline GUI
//!

use crate::app::ActionRequest;
use crate::components::{BooleanExpressionGui, HintText};
use crate::config::SharedConfig;
use crate::consts::DEFAULT_WINDOW_SIZES;
use crate::shortcuts::global_shortcuts;
use crate::spawn_transaction_no_commit_send_result;
use crate::windows::{Deleted, DeletedStatus};
use bool_tag_expr::BoolTagExpr;
use eframe::egui::{
    Align, CentralPanel, Context, DragValue, Id, Layout, RichText, Slider, Ui, Vec2, ViewportId,
};
use open_timeline_core::{Date, MAX_YEAR, MIN_YEAR, Name, OpenTimelineId, TimelineView};
use open_timeline_crud::{CrudError, FetchById};
use open_timeline_gui_core::{
    BreakOutWindow, CheckForUpdates, Draw, Reload, body_text_height, font_size, window_has_focus,
};
use open_timeline_gui_core::{EmptyConsideredInvalid, Shortcut, ShowRemoveButton};
use open_timeline_renderer::frontends::desktop_egui::OpenTimelineRendererEgui;
use open_timeline_renderer::{MAX_DATETIME_SCALE, MIN_DATETIME_SCALE, TimelineInteractionEvent};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, UnboundedSender};

/// View a timeline
pub struct TimelineViewGui {
    /// The ID of the timeline being viewed
    timeline_id: OpenTimelineId,

    /// The name of the timeline being viewed.
    timeline_name: Option<Name>,

    /// Send the ID of an `Entity` to be viewed
    tx_action_request: UnboundedSender<ActionRequest>,

    /// Receive reloaded data
    rx_reload: Option<Receiver<Result<TimelineView, CrudError>>>,

    /// Whether or not a reload has been requested
    requested_reload: bool,

    // TODO: we might want tags one day
    // tags: Tags,

    // TODO: might be nice to auto scroll to an entity/save location
    // dragged: Vec2,
    /// The renderer (engine frontend) that draws the timeline
    timeline_renderer: OpenTimelineRendererEgui,

    /// Whether the timeline has been deleted or not.  If it has been, the
    /// `Deleted` variant holds the `Instant` this window became aware of the
    /// fact.
    deleted_status: DeletedStatus,

    /// Whether this window should be closed or not
    wants_to_be_closed: bool,

    /// Database pool
    shared_config: SharedConfig,

    /// Whether or not the timeline controls should be shown
    show_controls: bool,

    // TODO: move all of the remaining into their own struct?
    /// Filter the timeline's entities using this bool expr
    bool_tag_expr_filter: BooleanExpressionGui,

    bool_tag_expr_filter_enabled: bool,

    ///
    start_date_limit: i64,
    start_date_limit_enabled: bool,
    end_date_limit: i64,
    end_date_limit_enabled: bool,
    datetime_scaling: f64,
    sticky_text: bool,
}

impl TimelineViewGui {
    // TODO: can we pass in a ReducedTimeline so that the name is displayed while
    // waiting for it to load?
    /// Create a new timeline viewing window
    pub fn new(
        shared_config: SharedConfig,
        ctx: &Context,
        tx_action_request: UnboundedSender<ActionRequest>,
        timeline_id: OpenTimelineId,
    ) -> Self {
        let bool_tag_expr_filter = BooleanExpressionGui::new(
            ShowRemoveButton::No,
            EmptyConsideredInvalid::No,
            HintText::Default,
        );

        let mut renderer = OpenTimelineRendererEgui::new(ctx);
        renderer.set_font_size_px(font_size(ctx) as f64);

        let mut timeline_view_gui = TimelineViewGui {
            timeline_id,
            timeline_name: None,
            tx_action_request,
            rx_reload: None,
            requested_reload: false,
            timeline_renderer: renderer,
            deleted_status: DeletedStatus::NotDeleted,
            wants_to_be_closed: false,
            shared_config,
            show_controls: true,
            bool_tag_expr_filter,
            bool_tag_expr_filter_enabled: false,
            start_date_limit: 1850,
            start_date_limit_enabled: false,
            end_date_limit: 2050,
            end_date_limit_enabled: false,
            datetime_scaling: 1.0,
            sticky_text: true,
        };
        timeline_view_gui.request_reload();
        timeline_view_gui
    }

    /// Get the ID of the timeline being viewed
    pub fn timeline_id(&self) -> OpenTimelineId {
        self.timeline_id
    }

    // TODO: don't want to do this every time
    // TODO: really shouldn't use .blocking_read()
    ///
    fn check_for_timeline_colour_changes(&mut self, ctx: &Context) {
        let colour_theme = self.shared_config.blocking_read().config.colour_theme;
        let timeline_colours = colour_theme.timeline_colours(ctx);
        self.timeline_renderer.set_colours(timeline_colours);
    }

    fn draw_filters(&mut self, ctx: &Context, ui: &mut Ui) -> (bool, bool) {
        ui.horizontal(|ui| {
            // Start date limit
            let start_checkbox_response =
                ui.checkbox(&mut self.start_date_limit_enabled, "Start Date Limit");
            let start_year_response = ui.add_enabled(
                self.start_date_limit_enabled,
                DragValue::new(&mut self.start_date_limit)
                    .speed(1)
                    .range(MIN_YEAR..=MAX_YEAR),
            );
            ui.separator();

            // Start date limit
            let end_checkbox_response =
                ui.checkbox(&mut self.end_date_limit_enabled, "End Date Limit");
            let end_year_response = ui.add_enabled(
                self.end_date_limit_enabled,
                DragValue::new(&mut self.end_date_limit)
                    .speed(1)
                    .range(MIN_YEAR..=MAX_YEAR),
            );
            ui.separator();

            // If value changed
            if start_year_response.changed() {
                self.start_date_limit_enabled = true;
                self.start_date_limit = self.start_date_limit.min(self.end_date_limit);
            }
            if end_year_response.changed() {
                self.end_date_limit_enabled = true;
                self.end_date_limit = self.start_date_limit.max(self.end_date_limit);
            }

            // Whether the date limits have changed
            let date_limits_changed = start_year_response.changed()
                || end_year_response.changed()
                || start_checkbox_response.changed()
                || end_checkbox_response.changed();

            // Filter by boolean tag expr
            let expr_filter_checkbox_response =
                ui.checkbox(&mut self.bool_tag_expr_filter_enabled, "Filter Entities");
            self.bool_tag_expr_filter.draw(ctx, ui);
            if self.bool_tag_expr_filter.changed() {
                self.bool_tag_expr_filter_enabled =
                    !self.bool_tag_expr_filter.expr().trim().is_empty();
            }

            // Whether the bool expr filtering has changed
            let tag_filter_changed =
                expr_filter_checkbox_response.changed() || self.bool_tag_expr_filter.changed();

            //
            (date_limits_changed, tag_filter_changed)
        })
        .inner
    }

    fn draw_controls(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Buttons
            // if ui.button("View Entity List").clicked() {
            //     // TODO
            // };

            // Stick text
            let sticky_text = ui.checkbox(&mut self.sticky_text, "Sticky Text");
            if sticky_text.changed() {
                self.timeline_renderer.set_sticky_text(self.sticky_text);
            }
            ui.separator();

            // Zoom
            if ui.button("Zoom Out").clicked() {
                self.timeline_renderer.zoom_out(1.1, 0.0, 0.0);
            }
            if ui.button("Zoom In").clicked() {
                self.timeline_renderer.zoom_in(1.1, 0.0, 0.0);
            }
            ui.separator();

            // x-scaling
            ui.label("Scale Date");
            ui.scope(|ui| {
                let slider = Slider::new(
                    &mut self.datetime_scaling,
                    MIN_DATETIME_SCALE..=MAX_DATETIME_SCALE,
                )
                .show_value(false);
                ui.spacing_mut().slider_width = ui.available_width();
                if ui.add(slider).changed() {
                    self.timeline_renderer
                        .set_datetime_scale(self.datetime_scaling);
                };
            });
        });
    }
}

impl Reload for TimelineViewGui {
    fn request_reload(&mut self) {
        if self.has_been_deleted() {
            return;
        }
        self.requested_reload = true;
        let timeline_id = self.timeline_id;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_reload = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        spawn_transaction_no_commit_send_result!(
            shared_config,
            bounded,
            tx,
            |transaction| async move { TimelineView::fetch_by_id(transaction, &timeline_id).await }
        );
    }

    fn check_reload_response(&mut self) {
        if let Some(rx) = self.rx_reload.as_mut() {
            match rx.try_recv() {
                Ok(result) => {
                    debug!("Recv timeline view reload response");
                    self.rx_reload = None;
                    self.requested_reload = false;
                    match result {
                        Ok(timeline) => {
                            self.timeline_name = Some(timeline.name().to_owned());
                            if let Some(entities) = timeline.entities() {
                                self.timeline_renderer.set_entities(entities.clone());
                                let (start, end) = self.timeline_renderer.start_and_end_dates();
                                self.start_date_limit = start as i64;
                                self.end_date_limit = end as i64;
                            }
                        }
                        Err(CrudError::IdNotInDb) => {
                            self.set_deleted_status(DeletedStatus::Deleted(Instant::now()))
                        }
                        Err(error) => warn!("Timeline view fetch error: {error}"),
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }
}

impl Deleted for TimelineViewGui {
    fn set_deleted_status(&mut self, deleted_status: DeletedStatus) {
        self.deleted_status = deleted_status;
    }

    fn deleted_status(&self) -> DeletedStatus {
        self.deleted_status
    }
}

impl CheckForUpdates for TimelineViewGui {
    fn check_for_updates(&mut self) {
        self.check_reload_response();
    }

    fn waiting_for_updates(&mut self) -> bool {
        let waiting = self.rx_reload.is_some();
        if waiting {
            info!("TimelineViewGui is waiting for updates");
        }
        waiting
    }
}

impl BreakOutWindow for TimelineViewGui {
    fn draw(&mut self, ctx: &Context) {
        // Handle shortcuts
        if window_has_focus(ctx) && Shortcut::close_window(ctx) {
            self.wants_to_be_closed = true;
        }

        // Check for global shortcuts
        global_shortcuts(ctx, &mut self.tx_action_request);

        // Draw
        CentralPanel::default().show(ctx, |ui| {
            // While waiting for the timeline to be fetched
            if self.requested_reload {
                ui.spinner();
                return;
            }

            // If the timeline is not in the database (anymore)
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

            // Title info (timeline name)
            let timeline_name = self.timeline_name.as_ref().unwrap().as_str();
            open_timeline_gui_core::Label::heading(ui, timeline_name);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Timeline").weak());

                // Toggle showing controls & filters
                let height = body_text_height(ui);
                ui.allocate_ui_with_layout(
                    Vec2::from([ui.available_width(), height]),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        ui.checkbox(&mut self.show_controls, "Show Controls");
                    },
                );
            });
            ui.separator();

            //
            if self.timeline_renderer.entity_count() == 0 {
                let text =
                    format!("The '{timeline_name}' timeline doesn't have any entities to show");
                open_timeline_gui_core::Label::weak(ui, &text);
                return;
            }

            if self.show_controls {
                // Timeline filters
                let (date_limits_changed, tag_filter_changed) = self.draw_filters(ctx, ui);
                ui.separator();

                // Controls
                self.draw_controls(ctx, ui);
                ui.separator();

                // Update timeline entity filter if appropriate
                if tag_filter_changed {
                    if self.bool_tag_expr_filter_enabled {
                        if let Ok(expr) = BoolTagExpr::from(self.bool_tag_expr_filter.expr()) {
                            self.timeline_renderer.set_tag_bool_expr_entity_filter(expr);
                        }
                    } else {
                        self.timeline_renderer.remove_tag_bool_expr_entity_filter();
                    }
                }

                // Update date limits if appropriate
                if date_limits_changed {
                    let start_limit = self
                        .start_date_limit_enabled
                        .then_some(Date::from(None, None, self.start_date_limit).unwrap());
                    let end_limit = self
                        .end_date_limit_enabled
                        .then_some(Date::from(None, None, self.end_date_limit).unwrap());
                    self.timeline_renderer
                        .set_date_limits(start_limit, end_limit);
                }

                // Get events
                for event in self.timeline_renderer.drain_interaction_events() {
                    match event {
                        TimelineInteractionEvent::SingleClick(entity_id)
                        | TimelineInteractionEvent::DoubleClick(entity_id)
                        | TimelineInteractionEvent::TripleClick(entity_id) => {
                            let _ = self.tx_action_request.send(ActionRequest::Entity(
                                crate::app::EntityOrTimelineActionRequest::ViewExisting(entity_id),
                            ));
                        }
                        _ => (),
                    }
                }
            }

            // Update colours
            self.check_for_timeline_colour_changes(ctx);

            // Draw the timeline
            self.timeline_renderer.draw(ctx, ui);
        });
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(
            DEFAULT_WINDOW_SIZES.timeline_view.width,
            DEFAULT_WINDOW_SIZES.timeline_view.height,
        )
    }

    fn viewport_id(&mut self) -> ViewportId {
        ViewportId(Id::from(format!("timeline_view_{}", self.timeline_id())))
    }

    fn title(&mut self) -> String {
        match self.timeline_name.as_ref() {
            None => String::from("View Timeline  -  [loading]"),
            Some(name) => format!("View Timeline • {}", name.as_str()),
        }
    }

    fn wants_to_be_closed(&mut self) -> bool {
        self.wants_to_be_closed
    }
}
