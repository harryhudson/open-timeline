// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything for GUI entities
//!

use crate::{common::ToOpenTimelineType, config::SharedConfig, consts::REMOVE_BUTTON_WIDTH};
use eframe::egui::{Context, Response, Ui};
use egui_dropdown::DropDownBox;
use open_timeline_core::{IsReducedType, Name, OpenTimelineId, ReducedEntities, ReducedEntity};
use open_timeline_crud::{CrudError, FetchByName, FetchByPartialName, Limit};
use open_timeline_gui_core::{
    Draw, ErrorStyle, Valid, ValidAsynchronous, ValidSynchronous, ValiditySynchronous,
    ValitityStatus, body_text_height, widget_x_spacing,
};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, error::TryRecvError};

use crate::spawn_transaction_no_commit_send_result;

#[derive(Debug)]
pub struct TimelineEntityGui {
    /// A random `OpenTimelineId` used by egui to uniquely identify the drop down menu
    search_results_dropdown_id: OpenTimelineId,

    /// The name of the entity
    name: String,

    /// Whether the user has requested the removal of this entity
    to_be_removed: bool,

    /// Results for the partial name search
    search_results: Vec<String>,

    /// Receiver from which we receive search results
    rx_search_results: Option<Receiver<Result<ReducedEntities, CrudError>>>,

    /// The validity of this timeline entity
    validity: ValitityStatus<ReducedEntity, CrudError>,

    /// Get this entity in its reduced from for CRUD operations
    as_reduced_entity: Option<ReducedEntity>,

    /// Database pool
    shared_config: SharedConfig,
}

impl TimelineEntityGui {
    /// Create new TimelineEntityGui
    pub fn new(shared_config: SharedConfig) -> Self {
        let mut new = Self {
            search_results_dropdown_id: OpenTimelineId::new(),
            name: String::new(),
            to_be_removed: false,
            search_results: Vec::new(),
            rx_search_results: None,
            validity: ValitityStatus::from(ValiditySynchronous::Valid, None),
            as_reduced_entity: None,
            shared_config,
        };
        new.update_validity();
        new
    }

    pub fn from_reduced_entity(shared_config: SharedConfig, reduced_entity: ReducedEntity) -> Self {
        Self {
            search_results_dropdown_id: OpenTimelineId::new(),
            name: reduced_entity.name().to_string(),
            to_be_removed: false,
            search_results: vec![],
            rx_search_results: None,
            validity: ValitityStatus::from(ValiditySynchronous::Valid, Some(Ok(()))),
            as_reduced_entity: Some(reduced_entity),
            shared_config,
        }
    }

    // TODO: make trait?
    /// Whether the user has requested the removal of this entity
    pub fn to_be_removed(&self) -> bool {
        self.to_be_removed
    }

    // TODO: nigh on identical to the code in entities.rs (use a macro to avoid generic hell)
    fn request_new_search_results(&mut self) {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_search_results = Some(rx);
        let partial_name = self.name.clone();
        let shared_config = Arc::clone(&self.shared_config);
        spawn_transaction_no_commit_send_result!(
            shared_config,
            bounded,
            tx,
            |transaction| async move {
                ReducedEntities::fetch_by_partial_name(transaction, Limit(10), &partial_name).await
            }
        );
    }

    // TODO: nigh on identical to the code in entities.rs (use a macro to avoid generic hell)
    fn check_for_search_result_response(&mut self) {
        if let Some(rx) = self.rx_search_results.as_mut() {
            if let Ok(Ok(results)) = rx.try_recv() {
                debug!("Recv timeline entity search response");
                self.rx_search_results = None;
                self.search_results = results
                    .names()
                    .into_iter()
                    .map(|name| name.to_string())
                    .collect();
            }
        }
    }

    /// Draw the results of the partial name search
    fn draw_search_results(&mut self, ui: &mut Ui) -> Response {
        let spacing = widget_x_spacing(ui);
        let input_height = body_text_height(ui);
        let input_width = ui.available_width() - spacing - REMOVE_BUTTON_WIDTH;
        ui.add_sized(
            [input_width, input_height],
            DropDownBox::from_iter(
                &mut self.search_results,
                self.search_results_dropdown_id,
                &mut self.name,
                |ui, text| ui.selectable_label(false, text),
            )
            .filter_by_input(false),
        )
    }
}

impl ErrorStyle for TimelineEntityGui {}

impl ValidSynchronous for TimelineEntityGui {
    fn is_valid_synchronous(&self) -> bool {
        self.validity.synchronous() == ValiditySynchronous::Valid
    }

    // TODO: basically the same as the GuiSubtimeline method
    fn update_validity_synchronous(&mut self) {
        match Name::from(self.name.clone()) {
            Ok(_) => self.validity.set_synchronous(ValiditySynchronous::Valid),
            Err(error) => self
                .validity
                .set_synchronous(ValiditySynchronous::Invalid(error.to_string())),
        };
    }

    fn validity_synchronous(&self) -> ValiditySynchronous {
        self.validity.synchronous()
    }
}

impl ValidAsynchronous for TimelineEntityGui {
    type Error = CrudError;

    fn check_for_asynchronous_validity_response(&mut self) {
        if let Some(rx) = self.validity.rx_asynchronous.as_mut() {
            match rx.try_recv() {
                Ok(msg) => {
                    debug!("Recv async validity response");
                    match msg {
                        Ok(reduced_entity) => {
                            self.as_reduced_entity = Some(reduced_entity);
                            self.validity.asynchronous = Some(Ok(()));
                        }
                        Err(error) => self.validity.asynchronous = Some(Err(error)),
                    }
                }
                Err(TryRecvError::Empty) => self.validity.asynchronous = None,
                Err(TryRecvError::Disconnected) => self.validity.rx_asynchronous = None,
            }
        }
    }

    fn is_valid_asynchronous(&self) -> Option<Result<(), Self::Error>> {
        self.validity.asynchronous.clone()
    }

    fn trigger_asynchronous_validity_update(&mut self) {
        self.validity.asynchronous = None;
        self.as_reduced_entity = None;
        let name = Name::from(self.name.clone()).unwrap();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.validity.rx_asynchronous = Some(rx);
        let shared_config = Arc::clone(&self.shared_config);
        spawn_transaction_no_commit_send_result!(
            shared_config,
            bounded,
            tx,
            |transaction| async move { ReducedEntity::fetch_by_name(transaction, &name).await }
        );
    }
}

impl Valid for TimelineEntityGui {}

impl ToOpenTimelineType<ReducedEntity> for TimelineEntityGui {
    // TODO: also a near copy
    fn to_opentimeline_type(&self) -> ReducedEntity {
        self.as_reduced_entity.clone().unwrap()
    }
}

impl Draw for TimelineEntityGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        self.check_for_asynchronous_validity_response();
        self.check_for_search_result_response();

        ui.horizontal(|ui| {
            ui.scope(|ui| {
                self.set_validity_styling(ctx, ui);

                // Draw current search results
                let input_box = self.draw_search_results(ui);

                // Request new search results
                if input_box.changed() || input_box.gained_focus() {
                    self.request_new_search_results();
                }

                // Update validity
                {
                    if input_box.lost_focus() {
                        self.update_validity();
                    };
                    if input_box.changed() {
                        self.update_validity();
                    }
                }
            });

            // Button always has same styling
            if open_timeline_gui_core::Button::remove(ui).clicked() {
                self.to_be_removed = true;
            }
        });
    }
}
