// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything needed to handle 1 boolean expression
//!

use crate::{common::ToOpenTimelineType, consts::REMOVE_BUTTON_WIDTH};
use bool_tag_expr::BoolTagExpr;
use eframe::egui::{Context, TextEdit, Ui};
use open_timeline_crud::CrudError;
use open_timeline_gui_core::{
    Draw, EmptyConsideredInvalid, ErrorStyle, ShowRemoveButton, Valid, ValidAsynchronous,
    ValidSynchronous, ValiditySynchronous, ValitityStatus, body_text_height, widget_x_spacing,
};

/// What hint text to show
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintText {
    None,
    Default,
}

/// GUI component for input a boolean tag expression
#[derive(Debug)]
pub struct BooleanExpressionGui {
    /// The bool expr that is editable by the user
    expr: String,

    /// The validity of the boolean expression.  This accounts for whether the
    /// bool expr is extant.
    validity: ValitityStatus<(), CrudError>,

    /// Whether or not to show the remove button.  The button is shown when
    /// editing a timeline, but not when used to filter a timeline's entities.
    show_remove_button: ShowRemoveButton,

    /// Whether an empty input should be considered invalid
    empty_considered_invalid: EmptyConsideredInvalid,

    // Hint text to show
    hint_text: HintText,

    /// Whether the expr has been changed by user input
    changed: bool,
}

impl BooleanExpressionGui {
    /// Create new `TimelineBooleanExpressionGui`.  This correctly sets the
    /// validity of the new component.
    pub fn new(
        show_remove_button: ShowRemoveButton,
        empty_considered_invalid: EmptyConsideredInvalid,
        hint_text: HintText,
    ) -> Self {
        let mut new = Self {
            expr: String::new(),
            validity: ValitityStatus::from(ValiditySynchronous::Valid, None),
            show_remove_button,
            hint_text,
            empty_considered_invalid,
            changed: false,
        };
        new.update_validity();
        new
    }

    /// Create new `TimelineBooleanExpressionGui` from `BoolTagExpr`.
    pub fn from_bool_tag_expr(
        show_remove_button: ShowRemoveButton,
        empty_considered_invalid: EmptyConsideredInvalid,
        hint_text: HintText,
        bool_tag_expr: BoolTagExpr,
    ) -> Self {
        let mut new = Self {
            expr: bool_tag_expr.to_boolean_expression(),
            validity: ValitityStatus::from(ValiditySynchronous::Valid, None),
            show_remove_button,
            hint_text,
            empty_considered_invalid,
            changed: false,
        };
        new.update_validity();
        new
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn expr(&self) -> &str {
        &self.expr
    }
}

impl ErrorStyle for BooleanExpressionGui {}

impl ValidSynchronous for BooleanExpressionGui {
    fn is_valid_synchronous(&self) -> bool {
        self.validity.synchronous() == ValiditySynchronous::Valid
    }

    // TODO: if the expr has no tags, then it's effectively empty - clear it, or
    // flag the user with an error
    fn update_validity_synchronous(&mut self) {
        if self.empty_considered_invalid == EmptyConsideredInvalid::No {
            if self.expr.trim().is_empty() {
                self.validity.set_synchronous(ValiditySynchronous::Valid);
                return;
            }
        }
        if let Err(error) = BoolTagExpr::from(self.expr.clone()) {
            self.validity
                .set_synchronous(ValiditySynchronous::Invalid(error.to_string()));
            return;
        }
        self.validity.set_synchronous(ValiditySynchronous::Valid);
    }

    fn validity_synchronous(&self) -> ValiditySynchronous {
        self.validity.synchronous().clone()
    }
}

impl ValidAsynchronous for BooleanExpressionGui {
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

impl Valid for BooleanExpressionGui {}

impl ToOpenTimelineType<BoolTagExpr> for BooleanExpressionGui {
    fn to_opentimeline_type(&self) -> BoolTagExpr {
        BoolTagExpr::from(self.expr.clone()).unwrap()
    }
}

impl Draw for BooleanExpressionGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.scope(|ui| {
            self.set_validity_styling(ctx, ui);

            // Calculations for layout
            let spacing = widget_x_spacing(ui);
            let input_height = body_text_height(ui);
            let input_width = if self.show_remove_button == ShowRemoveButton::Yes {
                ui.available_width() - spacing - REMOVE_BUTTON_WIDTH
            } else {
                ui.available_width()
            };

            // Display the text input
            let hint_text = match self.hint_text {
                HintText::None => "",
                HintText::Default => {
                    r#"Tag Boolean Expression (e.g. "british & (scientist | painter)")"#
                }
            };
            let input_box = ui.add_sized(
                [input_width, input_height],
                TextEdit::singleline(&mut self.expr)
                    .desired_width(input_width)
                    .hint_text(hint_text),
            );
            self.changed = input_box.changed();

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
    }
}
