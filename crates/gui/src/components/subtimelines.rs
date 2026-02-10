// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything needed to work with multiple subtimelines
//!

use crate::{
    common::ToOpenTimelineType, components::TimelineSubtimelineGui, config::SharedConfig,
    impl_is_valid_method_for_iterable, impl_valid_asynchronous_macro_never_called,
};
use eframe::egui::{Context, Ui};
use open_timeline_core::{IsReducedCollection, ReducedTimelines};
use open_timeline_gui_core::{
    Draw, ShowRemoveButton, Valid, ValidSynchronous, ValidityAsynchronous, ValiditySynchronous,
};
use std::sync::Arc;

/// Manages GUI state of a timeline's subtimelines
#[derive(Debug)]
pub struct TimelineSubtimelinesGui {
    /// All the GUI entity components
    subtimelines: Vec<TimelineSubtimelineGui>,

    /// Database pool
    shared_config: SharedConfig,
}

impl TimelineSubtimelinesGui {
    /// Create new TimelineSubtimelinesGui
    pub fn new(shared_config: SharedConfig) -> Self {
        Self {
            subtimelines: vec![],
            shared_config,
        }
    }

    pub fn add_empty_subtimeline(&mut self) {
        self.subtimelines.push(TimelineSubtimelineGui::new(
            Arc::clone(&self.shared_config),
            ShowRemoveButton::Yes,
        ));
    }

    pub fn from_reduced_timelines(
        shared_config: SharedConfig,
        orginal_timelines: Option<ReducedTimelines>,
    ) -> Self {
        let timelines = match orginal_timelines.clone() {
            None => Vec::new(),
            Some(timelines) => timelines
                .into_iter()
                .map(|timeline| {
                    TimelineSubtimelineGui::from_reduced_timeline(
                        Arc::clone(&shared_config),
                        ShowRemoveButton::Yes,
                        timeline,
                    )
                })
                .collect(),
        };
        Self {
            subtimelines: timelines,
            shared_config,
        }
    }
}

impl ToOpenTimelineType<Option<ReducedTimelines>> for TimelineSubtimelinesGui {
    fn to_opentimeline_type(&self) -> Option<ReducedTimelines> {
        let mut subtimelines = ReducedTimelines::new();
        for subtimeline in &self.subtimelines {
            let subtimeline = subtimeline.to_opentimeline_type();
            subtimelines.collection_mut().insert(subtimeline);
        }
        (!subtimelines.collection().is_empty()).then_some(subtimelines)
    }
}

impl ValidSynchronous for TimelineSubtimelinesGui {
    fn is_valid_synchronous(&self) -> bool {
        self.subtimelines
            .iter()
            .all(|subtimeline| subtimeline.is_valid_synchronous())
    }

    fn update_validity_synchronous(&mut self) {
        self.subtimelines
            .iter_mut()
            .for_each(|subtimeline| subtimeline.update_validity_synchronous())
    }

    fn validity_synchronous(&self) -> ValiditySynchronous {
        for subtimeline in &self.subtimelines {
            match subtimeline.validity_synchronous() {
                ValiditySynchronous::Invalid(error) => return ValiditySynchronous::Invalid(error),
                ValiditySynchronous::Valid => continue,
            }
        }
        ValiditySynchronous::Valid
    }
}

impl_valid_asynchronous_macro_never_called!(TimelineSubtimelinesGui);

impl Valid for TimelineSubtimelinesGui {
    fn validity(&self) -> ValidityAsynchronous {
        let validity: Vec<ValidityAsynchronous> = self
            .subtimelines
            .iter()
            .map(|subtimeline| subtimeline.validity())
            .collect();
        impl_is_valid_method_for_iterable!(validity)
    }

    fn update_validity(&mut self) {
        // Do nothing.  Components update their validity themselves.
        panic!()
    }
}

impl Draw for TimelineSubtimelinesGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        open_timeline_gui_core::Label::sub_heading(ui, "Subtimelines");

        // Display subtimelines
        if self.subtimelines.is_empty() {
            // "None" if no subtimelines
            open_timeline_gui_core::Label::none(ui);
        } else {
            // Draw subtimelines if there are any
            for subtimeline in &mut self.subtimelines {
                subtimeline.draw(ctx, ui);
            }

            // Remove any subtimelines that the user has requested to remove
            self.subtimelines
                .retain(|subtimeline| !subtimeline.to_be_removed());
        }
        ui.add_space(5.0);

        // Add subtimeline button
        if open_timeline_gui_core::Button::add(ui).clicked() {
            self.add_empty_subtimeline();
        }
    }
}
