// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything needed to work with a tag
//!

use crate::common::ToOpenTimelineType;
use crate::consts::REMOVE_BUTTON_WIDTH;
use bool_tag_expr::{Tag, TagError, TagName, TagValue};
use eframe::egui::{Context, TextEdit, Ui};
use open_timeline_crud::CrudError;
use open_timeline_gui_core::{
    Draw, ErrorStyle, ShowRemoveButton, Valid, ValidAsynchronous, ValidSynchronous,
    ValiditySynchronous, ValitityStatus, body_text_height, keyboard_input_cmd_and_enter,
    keyboard_input_cmd_and_k, widget_x_spacing,
};

// TODO: EntityTag and TimelineTag??

/// Which tag component to focus on.  This is used when creating a new input
/// row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagFocusRequestTarget {
    Name,
    Value,
}

/// Representation of the actions a user can take with respect to managing tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestedAction {
    Remove,
    AddNew(TagFocusRequestTarget),
}

/// GUI component for a tag
#[derive(Debug)]
pub struct TagGui {
    /// The tag name input buffer
    pub name: String,

    /// The tag value input buffer
    pub value: String,

    // TODO: improve this API?
    /// Which tag management action has been requested, if at all.  The user can
    /// add a new tag, or remove an existing one.
    pub requested_action: Option<RequestedAction>,

    /// Which tag component to request focus on.  This is used only when the tag
    /// input row is first created.
    ///
    /// Most of the time the value component will be focused on, but if the user
    /// uses the keyboard shortcut for a new tag while focused on a tag name,
    /// then the name component of the new tag will be focused on.
    component_to_focus_on: Option<TagFocusRequestTarget>,

    /// Whether or not to show the remove button.  The button is shown when
    /// editing a timeline or entity, but not when bulk editing a tag.
    show_remove_button: ShowRemoveButton,

    /// Everything needed for validation.
    validity: ValitityStatus<(), CrudError>,
}

impl TagGui {
    /// Create new `TagGui`
    pub fn new(
        show_remove_button: ShowRemoveButton,
        to_focus_on: Option<TagFocusRequestTarget>,
    ) -> Self {
        let mut new = Self {
            name: String::new(),
            value: String::new(),
            requested_action: None,
            component_to_focus_on: to_focus_on,
            show_remove_button,
            validity: ValitityStatus::from(ValiditySynchronous::Valid, None),
        };
        new.update_validity();
        new
    }

    /// Create a new `TagGui` from a `Tag`
    pub fn from_tag(tag: Tag, show_remove_button: ShowRemoveButton) -> Self {
        let name = match &tag.name {
            Some(tag) => tag.to_string(),
            None => String::new(),
        };
        let value = tag.value.to_string();
        Self {
            name,
            value,
            requested_action: None,
            component_to_focus_on: None,
            show_remove_button,
            validity: ValitityStatus::from(ValiditySynchronous::Valid, None),
        }
    }

    /// Whether the user has requested to remove the tag
    pub fn to_be_removed(&self) -> bool {
        matches!(self.requested_action, Some(RequestedAction::Remove))
    }

    pub fn invalid_msg(&self) -> String {
        self.validity.synchronous().invalid_msg()
    }
}

impl ValidSynchronous for TagGui {
    fn is_valid_synchronous(&self) -> bool {
        self.validity.synchronous() == ValiditySynchronous::Valid
    }

    fn update_validity_synchronous(&mut self) {
        // Tag name
        if !self.name.trim().is_empty() {
            match TagName::from(&self.name) {
                Ok(tag_name) => Some(tag_name),
                Err(error) => {
                    self.validity
                        .set_synchronous(ValiditySynchronous::Invalid(error.to_string()));
                    return;
                }
            };
        };

        // Tag value
        if let Err(error) = TagValue::from(&self.value) {
            let error_msg = match error {
                TagError::Empty => String::from("Tag name cannot be empty"),
                _ => error.to_string(),
            };
            self.validity
                .set_synchronous(ValiditySynchronous::Invalid(error_msg));
            return;
        };

        // Otherwise valid
        self.validity.set_synchronous(ValiditySynchronous::Valid);
    }

    fn validity_synchronous(&self) -> ValiditySynchronous {
        self.validity.synchronous()
    }
}

impl ValidAsynchronous for TagGui {
    type Error = CrudError;

    fn check_for_asynchronous_validity_response(&mut self) {
        //
    }

    fn is_valid_asynchronous(&self) -> Option<Result<(), Self::Error>> {
        Some(Ok(()))
    }

    fn trigger_asynchronous_validity_update(&mut self) {
        //
    }
}

impl Valid for TagGui {}

impl ErrorStyle for TagGui {}

impl ToOpenTimelineType<Tag> for TagGui {
    // TODO: reuse validation
    fn to_opentimeline_type(&self) -> Tag {
        let tag_name = (!self.name.trim().is_empty()).then(|| TagName::from(&self.name).unwrap());
        let tag_value = TagValue::from(&self.value).unwrap();
        Tag::from(tag_name, tag_value)
    }
}

impl Draw for TagGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        self.check_for_asynchronous_validity_response();

        // Sizings
        let spacing = widget_x_spacing(ui);
        let row_height = body_text_height(ui);
        let available_width = match self.show_remove_button {
            ShowRemoveButton::Yes => ui.available_width() - REMOVE_BUTTON_WIDTH - (spacing * 2.0),
            ShowRemoveButton::No => ui.available_width() - spacing,
        };
        let tag_component_input_width = available_width / 2.0;
        let tag_component_input_size = [tag_component_input_width, row_height];

        ui.horizontal(|ui| {
            let (name_input, value_input) = ui
                .scope(|ui| {
                    self.set_validity_styling(ctx, ui);

                    // Tag name & value inputs
                    let name_input = ui.add_sized(
                        tag_component_input_size,
                        TextEdit::singleline(&mut self.name),
                    );
                    let value_input = ui.add_sized(
                        tag_component_input_size,
                        TextEdit::singleline(&mut self.value),
                    );

                    // Update validity
                    {
                        if (name_input.lost_focus() || value_input.lost_focus())
                            && !(name_input.gained_focus() || value_input.gained_focus())
                        {
                            self.update_validity();
                        }

                        if name_input.changed() || value_input.changed() {
                            self.update_validity();
                        }
                    }

                    // Return input responses
                    (name_input, value_input)
                })
                .inner;

            // TODO: fix the width as REMOVE_BUTTON_WIDTH
            // "Remove" button
            if self.show_remove_button == ShowRemoveButton::Yes
                && open_timeline_gui_core::Button::remove(ui).clicked()
            {
                self.requested_action = Some(RequestedAction::Remove);
            }

            // Keyboard shortcuts
            {
                // Keyboard shortcut to add a new tag input (focus on name)
                if name_input.has_focus() && keyboard_input_cmd_and_enter(ctx) {
                    self.requested_action =
                        Some(RequestedAction::AddNew(TagFocusRequestTarget::Name));
                }

                // Keyboard shortcut to add a new tag input (focus on value)
                if value_input.has_focus() && keyboard_input_cmd_and_enter(ctx) {
                    self.requested_action =
                        Some(RequestedAction::AddNew(TagFocusRequestTarget::Value));
                }

                // Keyboard shortcut to remove tag
                if (value_input.has_focus() || name_input.has_focus())
                    && keyboard_input_cmd_and_k(ctx)
                {
                    self.requested_action = Some(RequestedAction::Remove);
                }
            }

            // If we have been instructed to focus on an input, do so
            if let Some(tag_component_focus_target) = self.component_to_focus_on.take() {
                match tag_component_focus_target {
                    TagFocusRequestTarget::Name => name_input.request_focus(),
                    TagFocusRequestTarget::Value => value_input.request_focus(),
                }
            }
        });
    }
}
