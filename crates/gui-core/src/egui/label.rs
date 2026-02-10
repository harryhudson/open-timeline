// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything needed to handle 1 boolean expression
//!

use eframe::egui::{self, Response, RichText, TextStyle, Ui};

/// Helpers for label drawing to an `egui` context
pub struct Label {}

impl Label {
    /// Draw the "None" label and return the response
    pub fn none(ui: &mut Ui) -> Response {
        let label = ui.label(RichText::new("None").weak());
        ui.add_space(5.0);
        label
    }

    /// Draw a strong label and return the response (short for
    /// `ui.label(RichText::new(text).strong())`)
    pub fn strong(ui: &mut Ui, text: &str) -> Response {
        ui.label(RichText::new(text).strong())
    }

    /// Draw a weak label and return the response (short for
    /// `ui.label(RichText::new(text).weak())`)
    pub fn weak(ui: &mut Ui, text: &str) -> Response {
        ui.label(RichText::new(text).weak())
    }

    /// Draw a heading label and return the response (short for
    /// `ui.heading(RichText::new(text))`)
    pub fn heading(ui: &mut Ui, text: &str) -> Response {
        ui.heading(RichText::new(text))
    }

    /// Draw a sub-heading and return the response (shortcut for
    /// `ui.label(RichText::new(text).heading())`)
    pub fn sub_heading(ui: &mut Ui, text: &str) -> Response {
        let heading_size = ui.style().text_styles[&TextStyle::Heading].size;
        let body_size = ui.style().text_styles[&TextStyle::Body].size;
        let size_difference = heading_size - body_size;
        let sub_heading_size = body_size + (size_difference / 3.0);
        let sub_heading = ui.add(egui::Label::new(
            RichText::new(text).size(sub_heading_size).strong(),
        ));
        ui.add_space(3.0);
        sub_heading
    }

    /// Write text for describing something and return the response (short
    /// for `ui.label(RichText::new(text).italics())`)
    pub fn description(ui: &mut Ui, text: &str) -> Response {
        ui.label(RichText::new(text).italics())
    }
}
