// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Desktop GUI app info panel
//!

use eframe::egui::{Align, Context, Layout, ScrollArea, Ui};
use egui_extras::{Column, TableBody, TableBuilder};
use open_timeline_gui_core::{Draw, body_text_height, widget_x_spacing};

/// The stats GUI panel in the main window
#[derive(Debug)]
pub struct AppInfoGui {}

impl AppInfoGui {
    /// Create a new app info GUI panel
    pub fn new() -> Self {
        Self {}
    }
}

impl Draw for AppInfoGui {
    fn draw(&mut self, _ctx: &Context, ui: &mut Ui) {
        // Sizes
        let row_height = body_text_height(ui);
        let spacing = widget_x_spacing(ui);
        let available_width = ui.available_width();
        let left_col_width = 150.0;
        let right_col_width = available_width - left_col_width - spacing;
        let right_col_width = right_col_width.max(0.0);

        ScrollArea::vertical().show(ui, |ui| {
            TableBuilder::new(ui)
                .id_salt("stats_table_body")
                .striped(true)
                .column(Column::exact(left_col_width))
                .column(Column::exact(right_col_width))
                .body(|mut body| {
                    // Programme name
                    draw_info_line(&mut body, row_height, "Name", |ui| {
                        ui.label("OpenTimeline");
                    });

                    // Programme version
                    draw_info_line(&mut body, row_height, "Version", |ui| {
                        ui.label(env!("CARGO_PKG_VERSION"));
                    });

                    // Link to website
                    draw_info_line(&mut body, row_height, "Website", |ui| {
                        ui.hyperlink("https://www.open-timeline.org");
                    });

                    // Information for reporting issues
                    draw_info_line(&mut body, row_height, "Report Issues", |ui| {
                        ui.scope(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label("Email us or report on Github at ");
                            ui.hyperlink("https://github.com/harryhudson/open-timeline/issues");
                        });
                    });

                    // Contact email
                    draw_info_line(&mut body, row_height, "Email", |ui| {
                        ui.label("all@open-timeline.org");
                    });

                    // OS of running system
                    draw_info_line(&mut body, row_height, "Operating System", |ui| {
                        let os = match std::env::consts::OS {
                            "windows" => "Windows",
                            "macos" => "macOS",
                            "linux" => "Linux",
                            other => other,
                        };
                        ui.label(os);
                    });

                    // Computer architecture of running system
                    draw_info_line(&mut body, row_height, "Architecture", |ui| {
                        let arch = match std::env::consts::ARCH {
                            "x86_64" => "x86_64",
                            "aarch64" => "arm64",
                            other => other,
                        };
                        ui.label(arch);
                    });

                    // License
                    draw_info_line(&mut body, row_height, "License", |ui| {
                        ui.label(
                            "GNU General Public License v3.0 or later (SPDX: GPL-3.0-or-later)",
                        );
                    });

                    // Link to source code
                    draw_info_line(&mut body, row_height, "Source Code", |ui| {
                        ui.hyperlink("https://github.com/harryhudson/open-timeline");
                    });

                    // Copyright information
                    draw_info_line(&mut body, row_height, "Copyright ©", |ui| {
                        ui.label("Harry Hudson");
                    });
                });
        });
    }
}

/// Draw a single row in a table, where the value can be any closure that draws
/// widgets
pub fn draw_info_line<F>(body: &mut TableBody, row_height: f32, name: &str, value: F)
where
    F: FnOnce(&mut Ui),
{
    body.row(row_height, |mut row| {
        row.col(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                open_timeline_gui_core::Label::strong(ui, name);
            });
        });
        row.col(|ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                value(ui);
            });
        });
    });
}
