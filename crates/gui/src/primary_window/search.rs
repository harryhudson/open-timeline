// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Full GUI search
//!

use crate::app::{ActionRequest, EntityOrTimelineActionRequest};
use crate::components::OpenTimelineButton;
use crate::components::{BooleanExpressionGui, HintText};
use crate::config::SharedConfig;
use crate::consts::{EDIT_BUTTON_WIDTH, VIEW_BUTTON_WIDTH};
use crate::spawn_transaction_no_commit_send_result;
use bool_tag_expr::BoolTagExpr;
use eframe::egui::{self, Align, Context, Layout, ScrollArea, TextEdit, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use open_timeline_core::{
    IsReducedCollection, IsReducedType, OpenTimelineId, ReducedEntities, ReducedEntity,
    ReducedTimeline, ReducedTimelines,
};
use open_timeline_crud::{CrudError, FetchByPartialNameAndBoolTagExpr, Limit};
use open_timeline_gui_core::{
    CheckForUpdates, Draw, EmptyConsideredInvalid, Reload, ShowRemoveButton, body_text_height,
    widget_x_spacing,
};
use std::sync::Arc;
use std::u32;
use tokio::sync::mpsc::{Receiver, UnboundedSender};

/// The maximum number of search results shown for each results section
const SEARCH_LIMIT: u32 = 75;

/// The search GUI panel in the main window
#[derive(Debug)]
pub struct SearchGui {
    /// Entity search terms & results
    entity_search: SearchPartialNameAndBoolTagExpr<ReducedEntities>,

    /// Timeline search terms & results
    timeline_search: SearchPartialNameAndBoolTagExpr<ReducedTimelines>,

    /// Used request new windows for editing and viewing timelines and entities
    tx_action_request: UnboundedSender<ActionRequest>,
}

impl SearchGui {
    /// Create a new search GUI panel manager
    pub fn new(
        shared_config: SharedConfig,
        tx_action_request: UnboundedSender<ActionRequest>,
    ) -> Self {
        let mut search = Self {
            entity_search: SearchPartialNameAndBoolTagExpr::<ReducedEntities>::new(Arc::clone(
                &shared_config,
            )),
            timeline_search: SearchPartialNameAndBoolTagExpr::<ReducedTimelines>::new(Arc::clone(
                &shared_config,
            )),
            tx_action_request,
        };
        search.request_reload();
        search
    }

    /// Display the entity search results fetched by partial name
    fn show_entity_search_results(&mut self, ui: &mut Ui, ctx: &Context) {
        let clicked = self.entity_search.show(ctx, ui);
        match clicked {
            None => (),
            Some(SearchResultButtonClicked::View(entity)) => {
                self.request_view_entity(ctx, ui, &entity)
            }
            Some(SearchResultButtonClicked::Edit(entity)) => {
                self.request_edit_entity(ctx, ui, &entity)
            }
        }
    }

    /// Display the timeline search results fetched by partial name
    fn show_timeline_search_results(&mut self, ui: &mut Ui, ctx: &Context) {
        let clicked = self.timeline_search.show(ctx, ui);
        match clicked {
            None => (),
            Some(SearchResultButtonClicked::View(timeline)) => {
                self.request_view_timeline(ctx, ui, &timeline)
            }
            Some(SearchResultButtonClicked::Edit(timeline)) => {
                self.request_edit_timeline(ctx, ui, &timeline)
            }
        }
    }

    /// Inform the main control loop that the user wants to edit an entity
    fn request_edit_entity(&mut self, _ctx: &Context, _ui: &mut Ui, entity: &ReducedEntity) {
        self.send_action_request(ActionRequest::Entity(
            EntityOrTimelineActionRequest::EditExisting(entity.id()),
        ));
    }

    /// Inform the main control loop that the user wants to view an entity
    fn request_view_entity(&mut self, _ctx: &Context, _ui: &mut Ui, entity: &ReducedEntity) {
        self.send_action_request(ActionRequest::Entity(
            EntityOrTimelineActionRequest::ViewExisting(entity.id()),
        ));
    }

    /// Inform the main control loop that the user wants to edit a timeline
    fn request_edit_timeline(&mut self, _ctx: &Context, _ui: &mut Ui, timeline: &ReducedTimeline) {
        self.send_action_request(ActionRequest::Timeline(
            EntityOrTimelineActionRequest::EditExisting(timeline.id()),
        ));
    }

    /// Inform the main control loop that the user wants to view a timeline
    fn request_view_timeline(&mut self, _ctx: &Context, _ui: &mut Ui, timeline: &ReducedTimeline) {
        self.send_action_request(ActionRequest::Timeline(
            EntityOrTimelineActionRequest::ViewExisting(timeline.id()),
        ));
    }

    /// Send an [`ActionRequest`] request to the main control loop
    fn send_action_request(&mut self, request: ActionRequest) {
        let _ = self.tx_action_request.send(request);
    }
}

impl Draw for SearchGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.columns(2, |columns| {
            // Timeline Column
            columns[0].vertical(|ui| {
                open_timeline_gui_core::Label::sub_heading(ui, "Timelines");

                // Button to create a new timeline
                if open_timeline_gui_core::Button::open_new(ui).clicked() {
                    self.send_action_request(ActionRequest::Timeline(
                        EntityOrTimelineActionRequest::CreateNew,
                    ));
                }

                ui.separator();
                draw_search_bars(ctx, ui, &mut self.timeline_search);
                ui.separator();
                self.show_timeline_search_results(ui, ctx);
            });

            // Entity column
            columns[1].vertical(|ui| {
                open_timeline_gui_core::Label::sub_heading(ui, "Entities");

                // Button to create a new entity
                if open_timeline_gui_core::Button::open_new(ui).clicked() {
                    self.send_action_request(ActionRequest::Entity(
                        EntityOrTimelineActionRequest::CreateNew,
                    ));
                }

                ui.separator();
                draw_search_bars(ctx, ui, &mut self.entity_search);
                ui.separator();
                self.show_entity_search_results(ui, ctx);
            });
        });
    }
}

impl CheckForUpdates for SearchGui {
    fn check_for_updates(&mut self) {
        self.timeline_search.check_reload_response();
        self.entity_search.check_reload_response();
    }

    fn waiting_for_updates(&mut self) -> bool {
        let waiting = self.timeline_search.rx_search_results.is_some()
            || self.entity_search.rx_search_results.is_some();
        if waiting {
            info!("SearchGui is waiting for updates");
        }
        waiting
    }
}

/// Draw search bars
fn draw_search_bars<T>(
    ctx: &Context,
    ui: &mut Ui,
    search_info: &mut SearchPartialNameAndBoolTagExpr<T>,
) where
    T: FetchByPartialNameAndBoolTagExpr + IsReducedCollection + Default + 'static,
{
    let changed = {
        // Search bar for searching by entity name
        let name_search_input = ui.add(
            TextEdit::singleline(&mut search_info.name_search)
                .desired_width(f32::INFINITY)
                .hint_text("Name"),
        );
        if name_search_input.changed() {
            search_info.name_search_active = true;
        }
        ui.add_space(5.0);

        // Search bar for searching by entity tag bool expr
        search_info.tag_boolean_expr_search.draw(ctx, ui);
        if search_info.tag_boolean_expr_search.changed() {
            search_info.tag_boolean_expr_search_active =
                !search_info.tag_boolean_expr_search.expr().trim().is_empty();
        }
        search_info.tag_boolean_expr_search.changed() || name_search_input.changed()
    };

    // Refresh search if needed
    if changed {
        search_info.request_reload();
    }
}

impl Reload for SearchGui {
    fn request_reload(&mut self) {
        self.entity_search.request_reload();
        self.timeline_search.request_reload();
    }

    fn check_reload_response(&mut self) {
        self.entity_search.check_reload_response();
        self.timeline_search.check_reload_response();
    }
}

/// Holds everything needed for searching by partial name and bool exprs
#[derive(Debug)]
struct SearchPartialNameAndBoolTagExpr<T>
where
    T: FetchByPartialNameAndBoolTagExpr + IsReducedCollection,
{
    /// Used to derive an ID for the GUI component
    gui_component_id_source: OpenTimelineId,

    /// Whether to search by partial name
    name_search_active: bool,

    /// The partial name to search by (if active)
    name_search: String,

    /// Whether to search by bool tag expr
    tag_boolean_expr_search_active: bool,

    /// The bool tag expr to search by (if active)
    tag_boolean_expr_search: BooleanExpressionGui,

    /// The search results
    search_results: T,

    /// Receive the search results
    rx_search_results: Option<Receiver<Result<T, CrudError>>>,

    /// Database pool
    shared_config: SharedConfig,
}

impl<T> SearchPartialNameAndBoolTagExpr<T>
where
    T: FetchByPartialNameAndBoolTagExpr + IsReducedCollection + Send + Default + 'static,
{
    /// Create a new `SearchPartialNameAndBoolTagExpr`
    fn new(shared_config: SharedConfig) -> Self {
        Self {
            gui_component_id_source: OpenTimelineId::new(),
            name_search_active: true,
            name_search: String::new(),
            tag_boolean_expr_search_active: false,
            tag_boolean_expr_search: BooleanExpressionGui::new(
                ShowRemoveButton::No,
                EmptyConsideredInvalid::No,
                HintText::Default,
            ),
            search_results: T::default(),
            rx_search_results: None,
            shared_config,
        }
    }

    /// Request a new search by just partial name
    fn request_new_search_by_partial_name(&mut self) {
        let partial_name = self.name_search.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_search_results = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        spawn_transaction_no_commit_send_result!(
            shared_config,
            bounded,
            tx,
            |transaction| async move {
                T::fetch_by_partial_name(transaction, Limit(SEARCH_LIMIT), &partial_name).await
            }
        );
    }

    /// Request a new search by just bool tag expr
    fn request_new_search_by_bool_tag_expr(&mut self) {
        let bool_tag_expr_result = BoolTagExpr::from(self.tag_boolean_expr_search.expr());
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_search_results = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        // TODO: can we use our spawn_block_needs_transaction_send_block_result_down_tx!() macro here? (add other with extra preamble arg?)
        tokio::spawn(async move {
            let bool_tag_expr = match bool_tag_expr_result {
                Ok(expr) => expr,
                Err(error) => {
                    let _ = tx.send(Err(CrudError::BoolExprParse(error))).await;
                    return;
                }
            };
            let result = async {
                let mut transaction = shared_config.read().await.db_pool.begin().await?;
                T::fetch_by_bool_tag_expr(&mut transaction, Limit(SEARCH_LIMIT), bool_tag_expr)
                    .await
            }
            .await;
            let _ = tx.send(result).await;
        });
    }

    /// Request a new search by both partial name & bool tag expr
    fn request_new_search_by_partial_name_and_bool_tag_expr(&mut self) {
        // Setup
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_search_results = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);

        // Partial name & bool tag expr
        let partial_name = self.name_search.clone();
        let bool_tag_expr_result = BoolTagExpr::from(self.tag_boolean_expr_search.expr());

        // TODO: can we use our spawn_block_needs_transaction_send_block_result_down_tx!() macro here? (add other with extra preamble arg?)
        tokio::spawn(async move {
            let bool_tag_expr = match bool_tag_expr_result {
                Ok(expr) => expr,
                Err(error) => {
                    let _ = tx.send(Err(CrudError::BoolExprParse(error))).await;
                    return;
                }
            };
            let result = async {
                let mut transaction = shared_config.read().await.db_pool.begin().await?;
                T::fetch_by_partial_name_and_bool_tag_expr(
                    &mut transaction,
                    Limit(SEARCH_LIMIT),
                    &partial_name,
                    bool_tag_expr,
                )
                .await
            }
            .await;
            let _ = tx.send(result).await;
        });
    }
}

impl<T> SearchPartialNameAndBoolTagExpr<T>
where
    T: FetchByPartialNameAndBoolTagExpr + IsReducedCollection + Clone + Default + 'static,
    <T as IsReducedCollection>::Item: Clone,
{
    // TODO: show matches count?
    // TODO: impl Draw?
    /// Draw search results to a table
    pub fn show(
        &mut self,
        _ctx: &Context,
        ui: &mut Ui,
    ) -> Option<SearchResultButtonClicked<<T as IsReducedCollection>::Item>> {
        ui.horizontal(|ui| {
            open_timeline_gui_core::Label::strong(ui, "Search By");
            let name_checkbox = ui.checkbox(&mut self.name_search_active, "Name");
            let expr_checkbox =
                ui.checkbox(&mut self.tag_boolean_expr_search_active, "Tag Bool Expr");
            if name_checkbox.changed() || expr_checkbox.changed() {
                self.request_reload();
            }
        });
        ui.separator();

        let available_width = ui.available_width();
        let table_height = ui.available_height();

        // Marshall search results
        let search_results = match (self.name_search_active, self.tag_boolean_expr_search_active) {
            (false, false) => None,
            _ => (!self.search_results.collection().is_empty())
                .then_some(self.search_results.clone()),
        };

        // Results
        match search_results {
            // If there are no search results
            None => {
                // Maintain the sizing of the search result area
                ui.allocate_ui(Vec2::from([available_width, table_height]), |ui| {
                    ui.set_min_size(Vec2::from([available_width, table_height]));
                    open_timeline_gui_core::Label::none(ui);
                });
                None
            }
            // If there are results to draw
            Some(search_results) => {
                // Sizes
                let row_height = body_text_height(ui);
                let spacing = widget_x_spacing(ui);
                let text_width =
                    available_width - VIEW_BUTTON_WIDTH - EDIT_BUTTON_WIDTH - (2.0 * spacing);
                let text_width = text_width.max(0.0);

                // Which to view or edit if there is one
                let mut to_view_or_edit = None;

                // Show the results in a table in a scrollable area
                ScrollArea::vertical()
                    .max_height(table_height)
                    .id_salt(format!("{}_scroll_area", self.gui_component_id_source))
                    .show(ui, |ui| {
                        ui.set_min_size(Vec2::from([available_width, table_height]));
                        TableBuilder::new(ui)
                            .id_salt(format!("{}_table", self.gui_component_id_source))
                            .striped(true)
                            .column(Column::exact(text_width).clip(true))
                            .column(Column::exact(EDIT_BUTTON_WIDTH))
                            .column(Column::exact(VIEW_BUTTON_WIDTH))
                            .body(|mut body| {
                                for (index, reduced_entity) in
                                    search_results.collection().iter().enumerate()
                                {
                                    if index as u32 > SEARCH_LIMIT {
                                        break;
                                    }
                                    body.row(row_height, |mut row| {
                                        // Name
                                        row.col(|ui| {
                                            let layout = Layout::left_to_right(Align::Center);
                                            let label =
                                                egui::Label::new(reduced_entity.name().as_str());
                                            ui.with_layout(layout, |ui| {
                                                ui.add(label.truncate());
                                            });
                                        });
                                        // Edit
                                        row.col(|ui| {
                                            if OpenTimelineButton::edit(ui).clicked() {
                                                to_view_or_edit =
                                                    Some(SearchResultButtonClicked::Edit(
                                                        reduced_entity.clone(),
                                                    ));
                                            }
                                        });
                                        // View
                                        row.col(|ui| {
                                            if OpenTimelineButton::view(ui).clicked() {
                                                to_view_or_edit =
                                                    Some(SearchResultButtonClicked::View(
                                                        reduced_entity.clone(),
                                                    ));
                                            }
                                        });
                                    });
                                }
                            });
                    });
                to_view_or_edit
            }
        }
    }
}

/// Used to indicate whether the edit or the view button was clicked.
pub enum SearchResultButtonClicked<T> {
    View(T),
    Edit(T),
}

impl<T> Reload for SearchPartialNameAndBoolTagExpr<T>
where
    T: FetchByPartialNameAndBoolTagExpr + IsReducedCollection + Default + 'static,
{
    fn request_reload(&mut self) {
        match (self.name_search_active, self.tag_boolean_expr_search_active) {
            (false, false) => {
                self.search_results.collection_mut().clear();
                self.rx_search_results = None;
            }
            (true, false) => self.request_new_search_by_partial_name(),
            (false, true) => self.request_new_search_by_bool_tag_expr(),
            (true, true) => self.request_new_search_by_partial_name_and_bool_tag_expr(),
        };
    }

    fn check_reload_response(&mut self) {
        if let Some(rx) = self.rx_search_results.as_mut() {
            if let Ok(data) = rx.try_recv() {
                self.rx_search_results = None;
                match data {
                    Ok(results) => self.search_results = results,
                    Err(_) => (),
                }
            }
        }
    }
}
