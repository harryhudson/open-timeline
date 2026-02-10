// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Those things used across the OpenTimeline GUI crate
//!

use eframe::egui::{Context, Ui};

/// Implementing types can be drawn to an egui context.
pub trait Draw {
    /// How to draw the type to an egui context.
    fn draw(&mut self, ctx: &Context, ui: &mut Ui);
}
