// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Keyboard shortcuts
//!

use crate::app::{ActionRequest, EntityOrTimelineActionRequest};
use eframe::egui::{Context, Key, KeyboardShortcut, Modifiers};
use tokio::sync::mpsc::UnboundedSender;

/// Helpers for handling OpenTimeline-specific keyboard shortcuts
pub struct OpenTimelineShortcut {}

impl OpenTimelineShortcut {
    /// Shortcut for creating a new entity window (Cmd + Shift + E)
    pub fn new_create_entity_window(ctx: &Context) -> bool {
        let modifiers = if cfg!(target_os = "macos") {
            Modifiers::MAC_CMD | Modifiers::SHIFT
        } else {
            Modifiers::CTRL | Modifiers::SHIFT
        };
        let new_create_entity_window_shortcut = KeyboardShortcut::new(modifiers, Key::E);
        let shortcut_used =
            ctx.input_mut(|i| i.consume_shortcut(&new_create_entity_window_shortcut));
        if shortcut_used {
            debug!("Create new entity window shortcut");
        }
        shortcut_used
    }

    /// Shortcut for creating a new timeline window (Cmd + Shift + T)
    pub fn new_create_timeline_window(ctx: &Context) -> bool {
        let modifiers = if cfg!(target_os = "macos") {
            Modifiers::MAC_CMD | Modifiers::SHIFT
        } else {
            Modifiers::CTRL | Modifiers::SHIFT
        };
        let create_timeline_window_shortcut = KeyboardShortcut::new(modifiers, Key::T);
        let shortcut_used = ctx.input_mut(|i| i.consume_shortcut(&create_timeline_window_shortcut));
        if shortcut_used {
            debug!("Create new timeline window shortcut");
        }
        shortcut_used
    }
}

/// Check for use of a global shortcut
pub fn global_shortcuts(ctx: &Context, tx_action_request: &mut UnboundedSender<ActionRequest>) {
    // New window for creating an entity
    if OpenTimelineShortcut::new_create_entity_window(ctx) {
        let _ = tx_action_request.send(ActionRequest::Entity(
            EntityOrTimelineActionRequest::CreateNew,
        ));
    }

    // New window for creating a timeline
    if OpenTimelineShortcut::new_create_timeline_window(ctx) {
        let _ = tx_action_request.send(ActionRequest::Timeline(
            EntityOrTimelineActionRequest::CreateNew,
        ));
    }
}
