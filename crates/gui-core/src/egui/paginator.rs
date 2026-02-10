// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Pagination controls
//!

use crate::{Draw, body_text_height, widget_x_spacing};
use eframe::egui::{self, Context, Ui};

/// Manages pagination and displays its controls
#[derive(Debug, Clone, Copy)]
pub struct Paginator {
    /// Holds which page (index) we're currently on.
    page_index: usize,

    /// The total number of items being paginated
    total_count: usize,

    /// The number of items per page
    items_per_page: usize,
}

impl Paginator {
    pub fn new(page_index: usize, total_count: usize, items_per_page: usize) -> Self {
        Paginator {
            page_index,
            total_count,
            items_per_page,
        }
    }

    pub fn page_index(&self) -> usize {
        self.page_index
    }

    pub fn set_page_index(&mut self, page_index: usize) {
        self.page_index = page_index
    }

    pub fn total_count(&self) -> usize {
        self.total_count
    }

    pub fn set_total_count(&mut self, total_count: usize) {
        self.total_count = total_count
    }

    pub fn items_per_page(&self) -> usize {
        self.items_per_page
    }

    pub fn set_items_per_page(&mut self, items_per_page: usize) {
        self.items_per_page = items_per_page
    }
}

impl Draw for Paginator {
    fn draw(&mut self, _ctx: &Context, ui: &mut Ui) {
        // Pages navigator sizings
        let row_height = body_text_height(ui);
        let x_spacing = widget_x_spacing(ui);
        let available_width = ui.available_width();
        let max_jump_button_width = 35.0;
        let jump_button_width = 30.0;
        let page_number_width = 25.0;
        let buttons_total_width = (2.0 * max_jump_button_width)
            + (2.0 * jump_button_width)
            + (5.0 * page_number_width)
            + (8.0 * x_spacing);

        // Subtract 2 x x_spacing to account for x_spacing around the outer buttons
        let space_remaining = available_width - buttons_total_width - (2.0 * x_spacing);
        let left_padding = space_remaining / 2.0;

        // Button sizes
        let button_max_jump_size = [max_jump_button_width, row_height];
        let button_jump_size = [jump_button_width, row_height];
        let button_page_number_size = [page_number_width, row_height];

        // Pagination calculations
        let max_page_index = self.total_count / self.items_per_page;

        // Draw the page navigation buttons
        ui.horizontal(|ui| {
            ui.add_space(left_padding);
            // Button to move to the first page
            let button = egui::Button::new("⏮");
            if ui.add_sized(button_max_jump_size, button).clicked() {
                self.page_index = 0;
            };

            // Button to move to the previous page
            if ui
                .add_sized(button_jump_size, egui::Button::new("⏴"))
                .clicked()
            {
                self.page_index = self.page_index.saturating_sub(1);
            };

            // Button for 2 pages before the current one
            match self.page_index {
                0 => ui.add_space(page_number_width + x_spacing),
                1 => ui.add_space(page_number_width + x_spacing),
                _ => {
                    let button = egui::Button::new(format!("{}", self.page_index - 1));
                    if ui.add_sized(button_page_number_size, button).clicked() {
                        self.page_index -= 2;
                    };
                }
            }

            // Button for 1 page before the current one
            match self.page_index {
                0 => ui.add_space(page_number_width + x_spacing),
                _ => {
                    let button = egui::Button::new(format!("{}", self.page_index));
                    if ui.add_sized(button_page_number_size, button).clicked() {
                        self.page_index -= 1;
                    };
                }
            };

            // Label for current page
            let label = egui::Label::new(format!("{}", self.page_index + 1));
            ui.add_sized(button_page_number_size, label);

            // Button for 1 page after the current one
            let button = egui::Button::new(format!("{}", self.page_index + 2));
            if (max_page_index - self.page_index) > 0 {
                if ui.add_sized(button_page_number_size, button).clicked() {
                    self.page_index += 1;
                };
            } else {
                ui.add_space(page_number_width + x_spacing)
            };

            // Button for 2 pages after the current one
            let button = egui::Button::new(format!("{}", self.page_index + 3));
            if (max_page_index - self.page_index) > 1 {
                if ui.add_sized(button_page_number_size, button).clicked() {
                    self.page_index += 2;
                };
            } else {
                ui.add_space(page_number_width + x_spacing)
            };

            // Button to move to the next page
            let button = egui::Button::new("⏵");
            if ui.add_sized(button_jump_size, button).clicked() {
                self.page_index = (self.page_index + 1).min(max_page_index);
            };

            // Button to move to the last page
            let button = egui::Button::new("⏭");
            if ui.add_sized(button_max_jump_size, button).clicked() {
                self.page_index = max_page_index;
            };
        });
    }
}
