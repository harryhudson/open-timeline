// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything needed to handle 1 boolean expression
//!

use crate::{
    ADD_SYMBOL, BEGIN_NEW_SYMBOL, CREATE_BUTTON_WIDTH, CREATE_SYMBOL, DELETE_BUTTON_WIDTH,
    DELETE_SYMBOL, REMOVE_BUTTON_WIDTH, REMOVE_SYMBOL, RESET_BUTTON_WIDTH, RESET_SYMBOL,
    UPDATE_BUTTON_WIDTH, UPDATE_SYMBOL, body_text_height,
};
use eframe::egui::{self, Response, RichText, Ui, Vec2};

/// Helpers for button drawing to an `egui` context
pub struct Button {}

impl Button {
    /// Draw the remove button and return the response
    pub fn remove(ui: &mut Ui) -> Response {
        let button_height = body_text_height(ui);
        ui.add_sized(
            [REMOVE_BUTTON_WIDTH, button_height],
            egui::Button::new(REMOVE_SYMBOL),
        )
    }

    /// Draw the delete button and return the response
    pub fn delete(ui: &mut Ui) -> Response {
        let button_height = body_text_height(ui);
        ui.add_sized(
            [DELETE_BUTTON_WIDTH, button_height],
            egui::Button::new(DELETE_SYMBOL),
        )
    }

    /// Draw the reset button and return the response
    pub fn reset(ui: &mut Ui) -> Response {
        let button_height = body_text_height(ui);
        ui.add_sized(
            [RESET_BUTTON_WIDTH, button_height],
            egui::Button::new(RESET_SYMBOL),
        )
    }

    /// Draw the create button and return the response
    pub fn create(ui: &mut Ui) -> Response {
        let button_height = body_text_height(ui);
        ui.add_sized(
            [CREATE_BUTTON_WIDTH, button_height],
            egui::Button::new(CREATE_SYMBOL),
        )
    }

    /// Draw the update button and return the response
    pub fn update(ui: &mut Ui) -> Response {
        let button_height = body_text_height(ui);
        ui.add_sized(
            [UPDATE_BUTTON_WIDTH, button_height],
            egui::Button::new(UPDATE_SYMBOL),
        )
    }

    /// Draw the begin new button and return the response
    pub fn open_new(ui: &mut Ui) -> Response {
        let button_height = body_text_height(ui);
        let button_width = ui.available_width();
        ui.add_sized(
            [button_width, button_height],
            egui::Button::new(BEGIN_NEW_SYMBOL),
        )
    }

    /// Helper to draw a tall button that fills the available GUI width
    pub fn tall_full_width(ui: &mut Ui, text: impl Into<RichText>) -> Response {
        ui.add_sized(
            Vec2::new(ui.available_width(), ui.spacing().interact_size.y * 2.0),
            egui::Button::new(text.into()),
        )
    }

    /// Draw a button to add a new row/instance of the thing and return the
    /// response
    pub fn add(ui: &mut Ui) -> Response {
        // Calculations for layout
        let button_height = body_text_height(ui);
        let button_width = ui.available_width();
        let button_size = [button_width, button_height];

        // Display the button for adding a bool expr
        let button = egui::Button::new(ADD_SYMBOL);
        ui.add_sized(button_size, button)
    }
}
