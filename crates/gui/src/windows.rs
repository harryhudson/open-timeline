// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! All of the pop-out/new OpenTimeline GUI windows
//!

mod app_colours;
mod entity_edit;
mod entity_view;
mod tag_edit;
mod tag_view;
mod timeline_edit;
mod timeline_view;

pub use app_colours::*;
pub use entity_edit::*;
pub use entity_view::*;
pub use tag_edit::*;
pub use tag_view::*;
pub use timeline_edit::*;
pub use timeline_view::*;

use crate::consts::{
    DEFAULT_NEW_WINDOW_X_OFFSET_FROM_MAIN_WINDOW, DEFAULT_NEW_WINDOW_Y_OFFSET_FROM_MAIN_WINDOW,
};
use eframe::egui::{Context, Pos2, Ui, Vec2, ViewportBuilder, ViewportCommand, ViewportId};
use open_timeline_gui_core::{BreakOutWindow, Draw, Reload};
use std::{collections::HashMap, hash::Hash, time::Instant};

pub type DeletedAtInstant = Instant;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DeletedStatus {
    Deleted(DeletedAtInstant),
    NotDeleted,
}

/// Implementing types can respond to the deletion of the data they're working
/// with
pub trait Deleted {
    fn set_deleted_status(&mut self, deleted_status: DeletedStatus);
    fn deleted_status(&self) -> DeletedStatus;

    fn draw_deleted_message(&mut self, _ctx: &Context, ui: &mut Ui) {
        if let DeletedStatus::Deleted(deleted_at) = self.deleted_status() {
            let elapsed_secs = deleted_at.elapsed().as_secs() as i32;
            let timeout_length_in_seconds = 5;
            let remaining_seconds = timeout_length_in_seconds - elapsed_secs;
            if remaining_seconds >= 1 {
                ui.label(format!("Will close in {remaining_seconds} secs"));
            }
        } else {
            //
        }
    }

    fn has_been_deleted(&self) -> bool {
        matches!(self.deleted_status(), DeletedStatus::Deleted(_))
    }
}

/// Holds information about a window (current just its position)
#[derive(Default, Debug, Clone)]
pub struct WindowInfo {
    pub offset: Pos2,
}

impl WindowInfo {
    pub fn new_with_offset(offset: Pos2) -> Self {
        debug!("New WindowInfo with offset: {offset:?}");
        WindowInfo {
            offset: offset
                + Vec2::new(
                    DEFAULT_NEW_WINDOW_X_OFFSET_FROM_MAIN_WINDOW,
                    DEFAULT_NEW_WINDOW_Y_OFFSET_FROM_MAIN_WINDOW,
                ),
        }
    }
}

/// All "break out" windows (those windows that are not the main window)
#[derive(Default)]
pub struct BreakOutWindows {
    windows: HashMap<ViewportId, (Box<dyn BreakOutWindow>, WindowInfo)>,
}

impl BreakOutWindows {
    pub fn insert(
        &mut self,
        ctx: &Context,
        main_window_pos: Option<Pos2>,
        mut window: Box<dyn BreakOutWindow>,
    ) {
        debug!("Adding new breakout window (title = '{}')", window.title());

        // Get the window's viewport ID
        let window_id = window.viewport_id();

        // If already open, bring it to the fore
        if self.windows.contains_key(&window_id) {
            ctx.send_viewport_cmd_to(window_id, ViewportCommand::Focus);

        // Otherwise create a new window (which will be brough to the fore for us)
        } else {
            let offset = main_window_pos.unwrap_or(Pos2::new(250.0, 150.0));
            self.windows
                .insert(window_id, (window, WindowInfo::new_with_offset(offset)));
        }
    }

    /// Update the viewport ID of any windows that have transformed from windows
    /// being used to create something into to windows being used to edit the
    /// thing they just created.
    ///
    /// Viewport IDs for windows used to create things are derived from a random
    /// `OpenTimelineId`.  Viewports for windows used to update things are derived
    /// (deterministically) from the thing they are showing/editing (derived
    /// from their ID if they have one).  When a thing is created, the window
    /// used to create it is transformed in-place to be a window for editing
    /// that very same thing.  We need to update the viewport ID to be derived
    /// from the ID of the newly created thing, rather than from a random
    /// `OpenTimelineId`.  We need to do this so that if we try to open a window to
    /// edit the thing, the currently open window will bought to the fore,
    /// rather than a new one opened.
    fn update_viewport_ids_if_needed(&mut self) {
        let mut new_map = HashMap::with_capacity(self.windows.len());
        for (_old_viewport_id, (mut window, window_info)) in self.windows.drain() {
            let new_viewport_id = window.viewport_id();
            new_map.insert(new_viewport_id, (window, window_info));
        }
        self.windows = new_map;
    }
}

impl Reload for BreakOutWindows {
    fn request_reload(&mut self) {
        self.update_viewport_ids_if_needed();
        for (window, _) in self.windows.values_mut() {
            window.request_reload()
        }
    }

    fn check_reload_response(&mut self) {
        // Nothing to check
    }
}

impl Draw for BreakOutWindows {
    fn draw(&mut self, ctx: &Context, _ui: &mut Ui) {
        let mut window_ids_to_close = Vec::new();
        let window_ids: Vec<ViewportId> = self.windows.keys().cloned().collect();
        for id in window_ids {
            let Some((window, window_info)) = self.windows.get_mut(&id) else {
                // TODO: panic here?
                continue;
            };
            let viewport = ViewportBuilder::default()
                .with_title(window.title())
                .with_position(window_info.offset)
                .with_inner_size(window.default_size());
            ctx.show_viewport_immediate(id, viewport, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) || window.wants_to_be_closed() {
                    window_ids_to_close.push(id);
                }
                if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                    window_info.offset = outer_rect.min;
                };
                window.draw(ctx);
            });
        }
        for id in window_ids_to_close {
            self.windows.remove(&id);
        }
    }
}
