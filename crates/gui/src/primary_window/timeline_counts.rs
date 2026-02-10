// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Desktop GUI timeline counts
//!

use crate::{
    app::{ActionRequest, EntityOrTimelineActionRequest},
    components::OpenTimelineButton,
    config::SharedConfig,
    consts::{EDIT_BUTTON_WIDTH, VIEW_BUTTON_WIDTH},
    spawn_transaction_no_commit_send_result,
};
use eframe::egui::{self, Align, Context, Layout, ScrollArea, TextEdit, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use open_timeline_core::HasIdAndName;
use open_timeline_crud::{CrudError, SortAlphabetically, SortByNumber, TimelineCounts};
use open_timeline_gui_core::{
    Draw, Paginator, Reload, body_text_height, widget_x_spacing, widget_y_spacing,
};
use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, UnboundedSender};

const UP_ARROW: &str = "⏶";
const DOWN_ARROW: &str = "⏷";
const UP_DOWN_ARROW: &str = "⏶⏷";

#[derive(Debug, Clone, Copy)]
struct TimelineCountsTableSizes {
    row_height: f32,
    count_width: f32,
    timeline_name_width: f32,
    edit_button_width: f32,
    view_button_width: f32,
    table_body_max_height: f32,
}

/// The timeline counts GUI panel in the main window
#[derive(Debug)]
pub struct TimelineCountsGui {
    /// The timeline counts (if they have been fetched). These are not sorted.
    timeline_counts: Option<TimelineCounts>,

    /// The timeline counts after they have been filtered by the search string.
    /// These are sorted.
    ///
    /// The `TimelineCounts` are owned because of difficulties with referencing other
    /// fields in the same struct.
    filtered_timeline_counts: Option<TimelineCounts>,

    // TODO: combine into a single enum
    /// How the count column should be ordered (if at all)
    count_ordering: Option<SortByNumber>,

    /// How the timeline name column should be ordered (if at all)
    name_ordering: Option<SortAlphabetically>,

    /// Receive up-to-date `TimelineCounts` after a reload requested
    rx_reload: Option<Receiver<Result<TimelineCounts, CrudError>>>,

    /// Whether a reload has been requested
    requested_reload: bool,

    /// Used request new timeline edit & timeline view windows
    tx_action_request: UnboundedSender<ActionRequest>,

    /// Used to filter the timelines (display timelines whose name or value contains the
    /// this string
    filter_text: String,

    /// Handles pagination
    paginator: Paginator,

    /// Database pool
    shared_config: SharedConfig,
}

impl TimelineCountsGui {
    /// Create a new timelines GUI panel manager
    pub fn new(
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
    ) -> Self {
        let mut timeline_count_gui = Self {
            timeline_counts: None,
            filtered_timeline_counts: None,
            count_ordering: None,
            name_ordering: None,
            rx_reload: None,
            requested_reload: false,
            tx_action_request,
            filter_text: String::new(),
            paginator: Paginator::new(0, 0, 100),
            shared_config,
        };
        timeline_count_gui.request_reload();
        timeline_count_gui
    }

    /// Update the order of the filtered timeline counts
    fn update_sort(&mut self) {
        if let Some(timeline_counts) = self.filtered_timeline_counts.as_mut() {
            if let Some(count_ordering) = &self.count_ordering {
                timeline_counts.sort_by_count(count_ordering);
            }
            if let Some(name_ordering) = &self.name_ordering {
                timeline_counts.sort_by_name(name_ordering);
            }
        }
    }

    /// Draw the table header row
    fn draw_table_header(
        &mut self,
        _ctx: &Context,
        ui: &mut Ui,
        table_sizes: TimelineCountsTableSizes,
    ) {
        let mut sort_needs_updating = false;
        begin_table(ui, "entity_timeline_counts_header", table_sizes).header(
            table_sizes.row_height,
            |mut row| {
                // Timeline entity counts
                row.col(|ui| {
                    let arrow = match self.count_ordering {
                        None => UP_DOWN_ARROW,
                        Some(SortByNumber::Ascending) => UP_ARROW,
                        Some(SortByNumber::Descending) => DOWN_ARROW,
                    };
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if open_timeline_gui_core::Label::sub_heading(
                            ui,
                            &format!("Entities {arrow}"),
                        )
                        .clicked()
                        {
                            self.name_ordering = None;
                            match self.count_ordering {
                                None => self.count_ordering = Some(SortByNumber::Ascending),
                                Some(SortByNumber::Ascending) => {
                                    self.count_ordering = Some(SortByNumber::Descending)
                                }
                                Some(SortByNumber::Descending) => self.count_ordering = None,
                            }
                            sort_needs_updating = true;
                        }
                    });
                });
                // Timeline names
                row.col(|ui| {
                    let arrow = match self.name_ordering {
                        None => UP_DOWN_ARROW,
                        Some(SortAlphabetically::AToZ) => UP_ARROW,
                        Some(SortAlphabetically::ZToA) => DOWN_ARROW,
                    };
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        if open_timeline_gui_core::Label::sub_heading(ui, &format!("Name {arrow}"))
                            .clicked()
                        {
                            self.count_ordering = None;
                            match self.name_ordering {
                                None => self.name_ordering = Some(SortAlphabetically::AToZ),
                                Some(SortAlphabetically::AToZ) => {
                                    self.name_ordering = Some(SortAlphabetically::ZToA)
                                }
                                Some(SortAlphabetically::ZToA) => self.name_ordering = None,
                            }
                            sort_needs_updating = true;
                        }
                    });
                });

                // Add space for edit & view buttons
                row.col(|_ui| {});
                row.col(|_ui| {});
            },
        );
        if sort_needs_updating {
            self.update_sort();
        }
    }

    /// Draw the table body
    fn draw_table_body(
        &mut self,
        _ctx: &Context,
        ui: &mut Ui,
        table_sizes: TimelineCountsTableSizes,
    ) {
        let Some(timeline_counts) = self.filtered_timeline_counts.as_ref() else {
            panic!()
        };

        // How a page/slice (too slow otherwise)
        let offset = (self.paginator.page_index()) * self.paginator.items_per_page();
        let upper_limit = timeline_counts
            .len()
            .min(offset + self.paginator.items_per_page());

        // offset..=upper_limit would overflow/be out of bounds
        let timeline_counts = &timeline_counts[offset..upper_limit];

        // Layouts
        let right_to_left = Layout::right_to_left(Align::Center);
        let left_to_right = Layout::left_to_right(Align::Center);

        ScrollArea::vertical()
            .max_height(table_sizes.table_body_max_height)
            .show(ui, |ui| {
                begin_table(ui, "entity_timeline_counts_body", table_sizes).body(|mut body| {
                    for timeline_count in timeline_counts {
                        let name = timeline_count.timeline().name().as_str();

                        body.row(table_sizes.row_height, |mut row| {
                            // Timeline entity count
                            row.col(|ui| {
                                ui.with_layout(right_to_left, |ui| {
                                    ui.add(
                                        egui::Label::new(format!(
                                            "{}",
                                            timeline_count.entity_count()
                                        ))
                                        .truncate(),
                                    );
                                });
                            });

                            // Timeline name
                            row.col(|ui| {
                                ui.with_layout(left_to_right, |ui| {
                                    ui.add(egui::Label::new(name).truncate());
                                });
                            });

                            // Button to request to edit the timeline
                            row.col(|ui| {
                                if OpenTimelineButton::edit(ui).clicked() {
                                    let _ = self.tx_action_request.send(ActionRequest::Timeline(
                                        EntityOrTimelineActionRequest::EditExisting(
                                            timeline_count.timeline().id().unwrap(),
                                        ),
                                    ));
                                }
                            });

                            // Button to request to view the timeline
                            row.col(|ui| {
                                if OpenTimelineButton::view(ui).clicked() {
                                    let _ = self.tx_action_request.send(ActionRequest::Timeline(
                                        EntityOrTimelineActionRequest::ViewExisting(
                                            timeline_count.timeline().id().unwrap(),
                                        ),
                                    ));
                                }
                            });
                        });
                    }
                });
            });
    }

    /// Update the filtered timeline counts
    fn update_filtered_timeline_counts(&mut self) {
        self.paginator.set_page_index(0);
        self.filtered_timeline_counts = self.timeline_counts.as_ref().map(|timeline_counts| {
            timeline_counts
                .into_iter()
                .filter(|timeline_count| {
                    timeline_count
                        .timeline()
                        .name()
                        .as_str()
                        .to_ascii_lowercase()
                        .contains(&self.filter_text.to_ascii_lowercase())
                })
                .cloned()
                .collect()
        });
        // If there are no timeline counts after filtering, convert to None
        self.filtered_timeline_counts = self
            .filtered_timeline_counts
            .take()
            .filter(|filtered_timeline_counts| !filtered_timeline_counts.is_empty());
        self.update_sort();
    }
}

impl Reload for TimelineCountsGui {
    fn request_reload(&mut self) {
        self.requested_reload = true;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_reload = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        spawn_transaction_no_commit_send_result!(
            shared_config,
            bounded,
            tx,
            |transaction| async move { TimelineCounts::fetch_all(transaction).await }
        );
    }

    fn check_reload_response(&mut self) {
        if let Some(rx) = self.rx_reload.as_mut() {
            match rx.try_recv() {
                Ok(msg) => match msg {
                    Ok(timeline_counts) => {
                        self.timeline_counts = Some(timeline_counts);
                        self.paginator.set_page_index(0);
                        self.update_filtered_timeline_counts();
                        self.rx_reload = None;
                        self.update_sort();
                        self.requested_reload = false;
                    }
                    Err(error) => eprintln!("Error fetching timeline counts: {error}"),
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => (),
            }
        }
    }
}

impl Draw for TimelineCountsGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        self.check_reload_response();

        // Input to filter by text
        let filter_input = ui.add(
            TextEdit::singleline(&mut self.filter_text)
                .desired_width(f32::INFINITY)
                .hint_text("Filter by timeline name"),
        );
        if filter_input.changed() {
            self.update_filtered_timeline_counts();
        }
        ui.separator();

        // Get number of timelines.  If there aren't any let the user know and return
        if let Some(timelines_counts) = &self.filtered_timeline_counts {
            self.paginator.set_total_count(timelines_counts.len());
        } else {
            open_timeline_gui_core::Label::none(ui);
            return;
        }

        // Sizes
        let available_width = ui.available_width();
        let available_height = ui.available_height();
        let row_height = body_text_height(ui);
        let x_spacing = widget_x_spacing(ui);
        let y_spacing = widget_y_spacing(ui);
        let count_width = 100.0;
        let table_max_height = available_height - (y_spacing * 3.0) - (row_height * 1.0);
        let table_body_max_height = table_max_height - (y_spacing * 1.0) - (row_height * 1.0);
        let timeline_name_width = available_width
            - count_width
            - EDIT_BUTTON_WIDTH
            - VIEW_BUTTON_WIDTH
            - (3.0 * x_spacing);

        // Stop underflows (cause egui to crash)
        let table_max_height = table_max_height.max(0.0);
        let table_body_max_height = table_body_max_height.max(0.0);
        let timeline_name_width = timeline_name_width.max(0.0);

        // Table sizes
        let table_sizes = TimelineCountsTableSizes {
            row_height,
            count_width,
            timeline_name_width,
            edit_button_width: EDIT_BUTTON_WIDTH,
            view_button_width: VIEW_BUTTON_WIDTH,
            table_body_max_height,
        };

        // Table
        ui.allocate_ui(Vec2::from([available_width, table_body_max_height]), |ui| {
            ui.set_min_size(Vec2::from([available_width, table_max_height]));
            self.draw_table_header(ctx, ui, table_sizes);
            self.draw_table_body(ctx, ui, table_sizes);
        });
        ui.separator();

        // Pagination controls
        self.paginator.draw(ctx, ui);
    }
}

/// Begin creating a table.  Used by both the table header and table body
/// drawing functions to ensure the columns match up
fn begin_table<'a>(
    ui: &'a mut Ui,
    id: &str,
    table_sizes: TimelineCountsTableSizes,
) -> TableBuilder<'a> {
    TableBuilder::new(ui)
        .id_salt(id)
        .striped(true)
        .column(Column::exact(table_sizes.count_width))
        .column(Column::exact(table_sizes.timeline_name_width))
        .column(Column::exact(table_sizes.edit_button_width))
        .column(Column::exact(table_sizes.view_button_width))
}
