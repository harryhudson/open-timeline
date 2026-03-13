// SPDX-License-Identifier: MIT

//!
//! Everything needed to work with a collection of tags
//!

use crate::{
    Draw, RequestedAction, ShowRemoveButton, TagFocusRequestTarget, TagGui, Valid,
    ValidAsynchronous, ValidSynchronous, ValidityAsynchronous, ValiditySynchronous, ValitityStatus,
};
use bool_tag_expr::{TagError, Tags};
use eframe::egui::{Context, Ui};

/// GUI component that manages & draws `TagGui`s
#[derive(Debug)]
pub struct TagsGui {
    /// All the tags held and shown to the user.
    tags: Vec<TagGui>,

    /// Tracks the overall validity of all the tags held.  All tags must be
    /// valid for this to say they are so.
    validity: ValitityStatus<(), String>,
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

    /// Get the GUI tags held
    pub fn tags(&self) -> &Vec<TagGui> {
        &self.tags
    }
}

impl TryInto<Option<Tags>> for &TagsGui {
    type Error = TagError;

    fn try_into(self) -> Result<Option<Tags>, Self::Error> {
        let mut tags = Tags::new();
        for tag in self.tags() {
            tags.insert(tag.try_into()?);
        }
        Ok((!tags.is_empty()).then_some(tags))
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

// Should never be called
impl ValidAsynchronous for TagsGui {
    type Error = String;

    fn is_valid_asynchronous(&self) -> Option<Result<(), Self::Error>> {
        // Do nothing.  Components update their validity themselves.
        panic!()
    }

    fn check_for_asynchronous_validity_response(&mut self) {
        // Do nothing.  Components update their validity themselves.
        panic!()
    }

    fn trigger_asynchronous_validity_update(&mut self) {
        // Do nothing.  Components update their validity themselves.
        panic!()
    }
}

impl Valid for TagsGui {
    fn validity(&self) -> ValidityAsynchronous {
        let validity: Vec<ValidityAsynchronous> =
            self.tags.iter().map(|tag| tag.validity()).collect();
        for validity in validity {
            match validity {
                ValidityAsynchronous::Invalid(error) => {
                    return ValidityAsynchronous::Invalid(error);
                }
                ValidityAsynchronous::Waiting => {
                    return ValidityAsynchronous::Waiting;
                }
                ValidityAsynchronous::Valid => continue,
            }
        }
        ValidityAsynchronous::Valid
    }

    fn update_validity(&mut self) {
        // Do nothing.  Components update their validity themselves.
        panic!()
    }
}

impl Draw for TagsGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        // Draw sub-heading
        crate::Label::sub_heading(ui, "Tags");

        // Track whether the user wants to add a new tag
        let mut add_new_tag = None;

        // Display tags
        if self.tags.is_empty() {
            // Tell the user that there are no tags
            crate::Label::none(ui);
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

        if crate::Button::add(ui).clicked() {
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
