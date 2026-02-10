// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything for GUI entities
//!

use crate::{
    common::ToOpenTimelineType, components::TimelineEntityGui, config::SharedConfig,
    impl_is_valid_method_for_iterable, impl_valid_asynchronous_macro_never_called,
};
use eframe::egui::{Context, Ui};
use open_timeline_core::{IsReducedCollection, ReducedEntities};
use open_timeline_gui_core::{
    Draw, Valid, ValidSynchronous, ValidityAsynchronous, ValiditySynchronous,
};
use std::sync::Arc;

/// Manages GUI state of a timeline's entities
#[derive(Debug)]
pub struct TimelineEntitiesGui {
    /// All the GUI entity components
    entities: Vec<TimelineEntityGui>,

    /// Database pool
    shared_config: SharedConfig,
}

impl TimelineEntitiesGui {
    /// Create new `TimelineEntitiesGui`
    pub fn new(shared_config: SharedConfig) -> Self {
        Self {
            entities: vec![],
            shared_config,
        }
    }

    /// Add a new GUI entity component to the list
    pub fn add_empty_entity(&mut self) {
        let shared_config = Arc::clone(&self.shared_config);
        self.entities.push(TimelineEntityGui::new(shared_config));
    }

    pub fn from_reduced_entities(
        shared_config: SharedConfig,
        original_entities: Option<ReducedEntities>,
    ) -> Self {
        let entities = match original_entities.clone() {
            None => Vec::new(),
            Some(tags) => tags
                .into_iter()
                .map(|entity| {
                    TimelineEntityGui::from_reduced_entity(Arc::clone(&shared_config), entity)
                })
                .collect(),
        };
        Self {
            entities,
            shared_config,
        }
    }
}

impl ValidSynchronous for TimelineEntitiesGui {
    fn is_valid_synchronous(&self) -> bool {
        self.entities
            .iter()
            .all(|entity| entity.is_valid_synchronous())
    }

    fn update_validity_synchronous(&mut self) {
        self.entities
            .iter_mut()
            .for_each(|entity| entity.update_validity_synchronous())
    }

    fn validity_synchronous(&self) -> ValiditySynchronous {
        for entity in &self.entities {
            match entity.validity_synchronous() {
                ValiditySynchronous::Invalid(error) => return ValiditySynchronous::Invalid(error),
                ValiditySynchronous::Valid => continue,
            }
        }
        ValiditySynchronous::Valid
    }
}

// TODO: same as for subtimelines
impl_valid_asynchronous_macro_never_called!(TimelineEntitiesGui);

impl Valid for TimelineEntitiesGui {
    fn validity(&self) -> ValidityAsynchronous {
        let validity: Vec<ValidityAsynchronous> = self
            .entities
            .iter()
            .map(|entity| entity.validity())
            .collect();
        impl_is_valid_method_for_iterable!(validity)
    }

    fn update_validity(&mut self) {
        // Do nothing.  Components update their validity themselves.
        panic!()
    }
}

impl ToOpenTimelineType<Option<ReducedEntities>> for TimelineEntitiesGui {
    fn to_opentimeline_type(&self) -> Option<ReducedEntities> {
        let mut entities = ReducedEntities::new();
        for entity in &self.entities {
            let entity = entity.to_opentimeline_type();
            entities.collection_mut().insert(entity);
        }
        (!entities.collection().is_empty()).then_some(entities)
    }
}

impl Draw for TimelineEntitiesGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        // Sub-heading
        open_timeline_gui_core::Label::sub_heading(ui, "Entities");

        // Entities
        if self.entities.is_empty() {
            // "None" if there aren't any entities
            open_timeline_gui_core::Label::none(ui);
        } else {
            // Draw the entities if there are some
            for entity in &mut self.entities {
                entity.draw(ctx, ui);
            }

            // If the user has requested an entity be remove from the list, do so
            self.entities.retain(|entity| !entity.to_be_removed());
        }
        ui.add_space(5.0);

        // Add entity button
        if open_timeline_gui_core::Button::add(ui).clicked() {
            self.add_empty_entity();
        }
    }
}
