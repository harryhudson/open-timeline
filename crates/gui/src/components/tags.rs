// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything needed to work with a collection of tags
//!

use crate::{
    common::ToOpenTimelineType,
    components::{RequestedAction, TagFocusRequestTarget, TagGui},
    impl_is_valid_method_for_iterable, impl_valid_asynchronous_macro_never_called,
};
use bool_tag_expr::Tags;
use eframe::egui::{Context, Ui};
use open_timeline_crud::CrudError;
use open_timeline_gui_core::{
    Draw, ShowRemoveButton, Valid, ValidSynchronous, ValidityAsynchronous, ValiditySynchronous,
    ValitityStatus,
};

/// GUI component that manages & draws `TagGui`s
#[derive(Debug)]
pub struct TagsGui {
    /// All the tags held and shown to the user.
    tags: Vec<TagGui>,

    /// Tracks the overall validity of all the tags held.  All tags must be
    /// valid for this to say they are so.
    validity: ValitityStatus<(), CrudError>,
}

impl TagsGui {
    /// Create a new `TagsGui`
    pub fn new() -> Self {
        Self {
            tags: vec![],
            validity: ValitityStatus::from(ValiditySynchronous::Valid, Some(Ok(()))),
        }
    }

    /// Add a new empty tag input to the list.  Passing along the focus target
    /// request
    fn add_empty_tag(&mut self, tag_focus_target: Option<TagFocusRequestTarget>) {
        self.tags
            .push(TagGui::new(ShowRemoveButton::Yes, tag_focus_target));
    }
}

impl ValidSynchronous for TagsGui {
    fn is_valid_synchronous(&self) -> bool {
        self.validity.synchronous() == ValiditySynchronous::Valid
    }

    fn update_validity_synchronous(&mut self) {
        for tag in &mut self.tags {
            if !tag.is_valid_synchronous() {
                self.validity
                    .set_synchronous(ValiditySynchronous::Invalid(tag.invalid_msg().to_owned()));
                return;
            }
        }
        // Otherwise valid
        self.validity.set_synchronous(ValiditySynchronous::Valid);
    }

    fn validity_synchronous(&self) -> ValiditySynchronous {
        self.validity.synchronous()
    }
}

impl_valid_asynchronous_macro_never_called!(TagsGui);

impl Valid for TagsGui {
    fn validity(&self) -> ValidityAsynchronous {
        let validity: Vec<ValidityAsynchronous> =
            self.tags.iter().map(|tag| tag.validity()).collect();
        impl_is_valid_method_for_iterable!(validity)
    }

    fn update_validity(&mut self) {
        // Do nothing.  Components update their validity themselves.
        panic!()
    }
}

impl ToOpenTimelineType<Option<Tags>> for TagsGui {
    fn to_opentimeline_type(&self) -> Option<Tags> {
        let opentimeline_tags: Tags = self
            .tags
            .iter()
            .map(|tag| tag.to_opentimeline_type())
            .collect();
        (!opentimeline_tags.is_empty()).then_some(opentimeline_tags)
    }
}

impl Draw for TagsGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        // Draw sub-heading
        open_timeline_gui_core::Label::sub_heading(ui, "Tags");

        // Track whether the user wants to add a new tag
        let mut add_new_tag = None;

        // Display tags
        if self.tags.is_empty() {
            // Tell the user that there are no tags
            open_timeline_gui_core::Label::none(ui);
        } else {
            // Draw each tag
            for tag in &mut self.tags {
                tag.draw(ctx, ui);

                // Act upon user request for a new tag row/input
                if let Some(RequestedAction::AddNew(target)) = tag.requested_action.as_ref() {
                    add_new_tag = Some(target.to_owned());
                    tag.requested_action = None;
                }
            }

            // If the user has requested a tag be removed from the list do so
            self.tags.retain(|tag| !tag.to_be_removed());
        }
        ui.add_space(5.0);

        // Add tag button

        if open_timeline_gui_core::Button::add(ui).clicked() {
            add_new_tag = Some(TagFocusRequestTarget::Value);
        }

        // If the user has requested a new tag row/input, add one
        if let Some(target) = add_new_tag.take() {
            self.add_empty_tag(Some(target));
        }
    }
}

impl From<Option<Tags>> for TagsGui {
    fn from(original_tags: Option<Tags>) -> Self {
        let tags = match original_tags.clone() {
            None => Vec::new(),
            Some(tags) => tags
                .into_iter()
                .map(|tag| TagGui::from_tag(tag, ShowRemoveButton::Yes))
                .collect(),
        };
        Self {
            tags,
            validity: ValitityStatus::from(ValiditySynchronous::Valid, Some(Ok(()))),
        }
    }
}
