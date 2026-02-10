// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Those things used across the OpenTimeline GUI crate
//!

use crate::Reload;
use eframe::egui::{Context, Vec2, ViewportId};

/// Implementing types are GUI windows
pub trait BreakOutWindow: Reload {
    fn draw(&mut self, ctx: &Context);
    fn default_size(&self) -> Vec2;
    fn viewport_id(&mut self) -> ViewportId;
    fn title(&mut self) -> String;

    // TODO: copied across all window (macro?)
    /// Implementing types (GUI windows) can request their closure.  This is done in
    /// response to the deletion of the underluing data they're working with, and
    /// thus implementing types must also implement the `Deleted` trait.
    fn wants_to_be_closed(&mut self) -> bool;
}
