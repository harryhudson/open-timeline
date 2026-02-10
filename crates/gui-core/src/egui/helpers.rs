// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Those things used across the OpenTimeline GUI crate
//!

use eframe::egui::{Context, TextStyle, Ui};

/// Whether the system is using Wayland or not
pub fn using_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}

// TODO: is this needed?
///
pub fn window_has_focus(ctx: &Context) -> bool {
    ctx.input(|i| i.focused)
}

/// Layout helper function (shortcut for `ui.spacing().interact_size.y`)
pub fn body_text_height(ui: &mut Ui) -> f32 {
    ui.spacing().interact_size.y
}

/// Layout helper function (shortcut for `ui.spacing().item_spacing.x`)
pub fn widget_x_spacing(ui: &mut Ui) -> f32 {
    ui.spacing().item_spacing.x
}

/// Layout helper function (shortcut for `ui.spacing().item_spacing.y`)
pub fn widget_y_spacing(ui: &mut Ui) -> f32 {
    ui.spacing().item_spacing.y
}

/// Get the current font size
pub fn font_size(ctx: &Context) -> f32 {
    ctx.style().text_styles[&TextStyle::Body].size
}
