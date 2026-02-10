// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything needed to handle 1 boolean expression
//!

use crate::consts::{EDIT_BUTTON_WIDTH, EDIT_SYMBOL, VIEW_SYMBOL};
use eframe::egui::{self, Response, RichText, Ui};
use open_timeline_gui_core::body_text_height;

pub struct OpenTimelineButton {}

impl OpenTimelineButton {
    /// Draw an edit button and return the response
    pub fn edit(ui: &mut Ui) -> Response {
        let button_height = body_text_height(ui);
        ui.add_sized(
            [EDIT_BUTTON_WIDTH, button_height],
            egui::Button::new(RichText::new(EDIT_SYMBOL)),
        )
    }

    /// Draw a view button and return the response
    pub fn view(ui: &mut Ui) -> Response {
        let button_height = body_text_height(ui);
        ui.add_sized(
            [EDIT_BUTTON_WIDTH, button_height],
            egui::Button::new(RichText::new(VIEW_SYMBOL)),
        )
    }
}
