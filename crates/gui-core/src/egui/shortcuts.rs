// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Keyboard shortcuts
//!

use eframe::egui::{Context, Key};

/// Helpers for handling keyboard shortcuts
pub struct Shortcut {}

impl Shortcut {
    pub fn save(ctx: &Context) -> bool {
        keyboard_input_cmd_and_s(ctx)
    }

    pub fn close_window(ctx: &Context) -> bool {
        keyboard_input_cmd_and_w(ctx)
    }
}

/// Has the user pressed `cmd` + `enter`
pub fn keyboard_input_cmd_and_enter(ctx: &Context) -> bool {
    ctx.input(|i| i.key_pressed(Key::Enter) && (i.modifiers.mac_cmd || i.modifiers.command))
}

/// Has the user pressed `cmd` + `k`
pub fn keyboard_input_cmd_and_k(ctx: &Context) -> bool {
    ctx.input(|i| {
        i.key_pressed(Key::K) && i.modifiers.shift && (i.modifiers.mac_cmd || i.modifiers.command)
    })
}

/// Has the user pressed `cmd` + `s`
fn keyboard_input_cmd_and_s(ctx: &Context) -> bool {
    ctx.input(|i| i.key_pressed(Key::S) && (i.modifiers.mac_cmd || i.modifiers.command))
}

/// Has the user pressed `cmd` + `s`
fn keyboard_input_cmd_and_w(ctx: &Context) -> bool {
    ctx.input(|i| i.key_pressed(Key::W) && (i.modifiers.mac_cmd || i.modifiers.command))
}
