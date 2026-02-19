// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Desktop GUI database stats
//!

use crate::config::SharedConfig;
use crate::spawn_transaction_no_commit_send_result;
use eframe::egui::{Align, Context, Layout, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};
use open_timeline_crud::{CrudError, DatabaseRowCount};
use open_timeline_gui_core::{CheckForUpdates, Draw, Reload, body_text_height, widget_x_spacing};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::error::TryRecvError;

/// The stats GUI panel in the main window
#[derive(Debug)]
pub struct StatsGui {
    /// Holds the row counts of each of the tables in the database.
    table_row_counts: Option<DatabaseRowCount>,

    /// Receive up-to-date row counts.
    rx_reload: Option<Receiver<Result<DatabaseRowCount, CrudError>>>,

    /// Whether or not a reload has been requested (automatically done in
    /// response to a successful CRUD operation being executed elsewhere in the
    /// application)
    requested_reload: bool,

    /// Database pool
    shared_config: SharedConfig,
}

impl StatsGui {
    /// Create a new stats GUI panel manager
    pub fn new(shared_config: SharedConfig) -> Self {
        let mut stats_gui = Self {
            table_row_counts: None,
            rx_reload: None,
            requested_reload: false,
            shared_config,
        };
        stats_gui.request_reload();
        stats_gui
    }
}

impl Reload for StatsGui {
    fn request_reload(&mut self) {
        self.requested_reload = true;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_reload = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        spawn_transaction_no_commit_send_result!(
            shared_config,
            bounded,
            tx,
            |transaction| async move {
                // TODO: renmae this and tag_counts.rs!
                DatabaseRowCount::all(transaction).await
            }
        );
    }

    fn check_reload_response(&mut self) {
        if let Some(rx) = self.rx_reload.as_mut() {
            match rx.try_recv() {
                Ok(msg) => {
                    debug!("Recv database row count response");
                    match msg {
                        Ok(row_counts) => {
                            self.table_row_counts = Some(row_counts);
                            self.rx_reload = None;
                            self.requested_reload = false;
                        }
                        Err(_) => todo!(),
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }
}

impl Draw for StatsGui {
    fn draw(&mut self, _ctx: &Context, ui: &mut Ui) {
        // Display stats
        if let Some(row_counts) = self.table_row_counts.as_ref() {
            // Sizes
            let row_height = body_text_height(ui);
            let spacing = widget_x_spacing(ui);
            let available_width = ui.available_width();
            let count_width = 100.0;
            let table_name_width = available_width - count_width - spacing;
            let table_name_width = table_name_width.max(0.0);

            let counts = vec![
                (row_counts.entities, "Entities"),
                (row_counts.entity_tags, "Entity Tags"),
                (row_counts.timelines, "Timelines"),
                (row_counts.subtimelines, "Subtimelines"),
                (row_counts.timeline_entities, "Timeline Entities"),
                (row_counts.timeline_tags, "Timeline Tags"),
            ];

            // Draw the table header (column names)
            TableBuilder::new(ui)
                .id_salt("stats_table_header")
                .striped(true)
                .column(Column::exact(count_width))
                .column(Column::exact(table_name_width))
                .header(row_height, |mut row| {
                    row.col(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            open_timeline_gui_core::Label::sub_heading(ui, "Count");
                        });
                    });
                    row.col(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            open_timeline_gui_core::Label::sub_heading(ui, "Type");
                        });
                    });
                });

            // Draw the table content
            ScrollArea::vertical().show(ui, |ui| {
                TableBuilder::new(ui)
                    .id_salt("stats_table_body")
                    .striped(true)
                    .column(Column::exact(count_width))
                    .column(Column::exact(table_name_width))
                    .body(|mut body| {
                        for count in counts {
                            body.row(row_height, |mut row| {
                                // Display the row count for the database table
                                row.col(|ui| {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(format!("{}", count.0))
                                    });
                                });

                                // Display the name of the database table
                                row.col(|ui| {
                                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                        ui.label(count.1)
                                    });
                                });
                            });
                        }
                    });
            });
        }
    }
}

impl CheckForUpdates for StatsGui {
    fn check_for_updates(&mut self) {
        self.check_reload_response();
    }

    fn waiting_for_updates(&mut self) -> bool {
        let waiting = self.rx_reload.is_some();
        if waiting {
            info!("StatsGui is waiting for updates");
        }
        waiting
    }
}
