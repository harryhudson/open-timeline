// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! GUI component for drawing statuses
//!

use eframe::egui::{Response, Ui};

/// Implementing types represent the status of something and can be drawn to an
/// `egui` context
pub trait DisplayStatus {
    fn status_display(&self, ui: &mut Ui) -> Response;
}

/// Used to show statuses
pub struct GuiStatus {}

impl GuiStatus {
    /// Draw the given status
    pub fn display(ui: &mut Ui, status: &impl DisplayStatus) {
        ui.horizontal(|ui| {
            crate::Label::strong(ui, "Status");
            status.status_display(ui)
        });
    }
}
