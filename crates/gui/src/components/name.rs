// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything needed to work with a name
//!

use crate::common::ToOpenTimelineType;
use crate::config::SharedConfig;
use crate::spawn_transaction_no_commit_send_result;
use eframe::egui::{Context, TextEdit, Ui};
use open_timeline_core::Name;
use open_timeline_crud::{CrudError, is_entity_name_in_db, is_timeline_name_in_db};
use open_timeline_gui_core::{
    Draw, ErrorStyle, Valid, ValidAsynchronous, ValidSynchronous, ValiditySynchronous,
    ValitityStatus,
};
use std::sync::Arc;
use tokio::sync::mpsc::error::TryRecvError;

/// Represents whether we're working with a timeline or entity name.
#[derive(Debug)]
pub enum EntityOrTimeline {
    Timeline,
    Entity,
}

/// Representation of whether the name is being created or edited.  If it is
/// being edited the name as it appears in the database is held so that
/// validation doesn't think the name is already taken.
#[derive(Debug, Clone, PartialEq)]
pub enum CreateOrEditName {
    Edit(Name),
    Create,
}

/// GUI component for inputing an entity or timeline name
#[derive(Debug)]
pub struct NameGui {
    /// The input buffer
    pub name: String,

    /// Whether this is the name of an entity or timeline.  This is used for
    /// validating against the database.
    entity_or_timeline: EntityOrTimeline,

    /// Whether the name is being create or edited.  This is used for validating
    /// against the databse.
    creating_or_editing: CreateOrEditName,

    /// Everything needed for validation.
    validity: ValitityStatus<bool, CrudError>,

    /// Database pool
    shared_config: SharedConfig,
}

impl NameGui {
    /// Create new NameGui
    pub fn new(shared_config: SharedConfig, entity_or_timeline: EntityOrTimeline) -> Self {
        let mut new = Self {
            name: String::new(),
            entity_or_timeline,
            creating_or_editing: CreateOrEditName::Create,
            validity: ValitityStatus::from(ValiditySynchronous::Valid, None),
            shared_config,
        };
        new.update_validity();
        new
    }

    pub fn from_name(
        shared_config: SharedConfig,
        entity_or_timeline: EntityOrTimeline,
        name: Name,
    ) -> Self {
        Self {
            name: name.to_string(),
            entity_or_timeline,
            creating_or_editing: CreateOrEditName::Edit(name.clone()),
            validity: ValitityStatus::from(ValiditySynchronous::Valid, Some(Ok(()))),
            shared_config,
        }
    }
}

impl ErrorStyle for NameGui {}

impl ValidSynchronous for NameGui {
    fn is_valid_synchronous(&self) -> bool {
        self.validity.synchronous() == ValiditySynchronous::Valid
    }

    fn update_validity_synchronous(&mut self) {
        debug!("Updating name validity");
        let sync_validity = match Name::from(self.name.clone()) {
            Ok(_) => ValiditySynchronous::Valid,
            Err(error) => ValiditySynchronous::Invalid(error.to_string()),
        };
        self.validity.set_synchronous(sync_validity);
    }

    fn validity_synchronous(&self) -> ValiditySynchronous {
        self.validity.synchronous().clone()
    }
}

impl ValidAsynchronous for NameGui {
    type Error = CrudError;

    fn check_for_asynchronous_validity_response(&mut self) {
        if let Some(rx) = self.validity.rx_asynchronous.as_mut() {
            match rx.try_recv() {
                Ok(msg) => {
                    debug!("Recv asynchronous validity response");
                    self.validity.rx_asynchronous = None;
                    // TODO: return an enum rather than bool for readability
                    match msg {
                        Ok(false) => self.validity.asynchronous = Some(Ok(())),
                        // TODO: use a different CrudError
                        Ok(true) => self.validity.asynchronous = Some(Err(CrudError::Name)),
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
        debug!("Triggering name async validity update");

        // Check if the name is as it is saved in the database (in which case it
        // is valid)
        let current_name = Name::from(self.name.clone()).unwrap();
        if let CreateOrEditName::Edit(name_editing) = &self.creating_or_editing {
            if current_name == *name_editing {
                // It is unchanged and is thus valid
                self.validity.asynchronous = Some(Ok(()));
                return;
            }
        }

        self.validity.asynchronous = None;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.validity.rx_asynchronous = Some(rx);

        let shared_config = Arc::clone(&self.shared_config);
        match &self.entity_or_timeline {
            EntityOrTimeline::Entity => {
                spawn_transaction_no_commit_send_result!(
                    shared_config,
                    bounded,
                    tx,
                    |transaction| async move {
                        is_entity_name_in_db(transaction, &current_name.clone()).await
                    }
                );
            }
            EntityOrTimeline::Timeline => {
                spawn_transaction_no_commit_send_result!(
                    shared_config,
                    bounded,
                    tx,
                    |transaction| async move {
                        is_timeline_name_in_db(transaction, &current_name.clone()).await
                    }
                );
            }
        }
    }
}

impl Valid for NameGui {}

impl Draw for NameGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        self.check_for_asynchronous_validity_response();

        // Draw sub heading
        open_timeline_gui_core::Label::sub_heading(ui, "Name");

        ui.scope(|ui| {
            self.set_validity_styling(ctx, ui);

            // Draw input box
            let input_box =
                ui.add(TextEdit::singleline(&mut self.name).desired_width(f32::INFINITY));

            // Update validity
            if input_box.changed() {
                debug!("Name input changed");
                self.update_validity();
            }
        });
    }
}

impl ToOpenTimelineType<Name> for NameGui {
    fn to_opentimeline_type(&self) -> Name {
        Name::from(self.name.clone()).unwrap()
    }
}
