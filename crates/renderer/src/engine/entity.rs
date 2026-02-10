// SPDX-License-Identifier: MIT

//!
//! Entity
//!

use crate::{
    Colour, FilledBox, MeasuredLayoutParams, Point, PositionAndSize, ScalableLayoutParams, TextOut,
    TextWorking, TimelineColours, TimelineDateRange, colours::Colours,
};
use bool_tag_expr::BoolTagExpr;
use open_timeline_core::{Date, Entity, HasIdAndName};
use serde::Serialize;
use std::fmt::Debug;

/// Information needed to draw an [`Entity`] on a timeline (for use outisde of
/// the engine)
#[derive(Debug, Clone, Serialize)]
pub struct EntityOut {
    pub entity: Entity,
    pub text: TextOut,
    pub text_box: FilledBox,
    pub date_box: FilledBox,
}

impl From<WorkingEntity> for EntityOut {
    fn from(value: WorkingEntity) -> Self {
        EntityOut {
            entity: value.entity,
            text: TextOut {
                top_left: value.text.top_left,
                text: value.text.text,
                colour: value.text.colour,
                font_size: value.text.font_size,
            },
            text_box: value.text_box,
            date_box: value.date_box,
        }
    }
}

/// Information needed when working/calculating with an entity (for internal use
/// by the engine)
#[derive(Debug, Clone, Serialize)]
pub(crate) struct WorkingEntity {
    // For external drawing
    pub entity: Entity,
    pub text: TextWorking,
    pub text_box: FilledBox,
    pub date_box: FilledBox,

    // For use by engine
    pub is_hovered_over: bool,
    pub is_hightlighted: bool,
    pub is_selected: bool,

    is_filtered_out_by_date_range: bool,
    is_filtered_out_by_bool_expr: bool,

    row: usize,

    // Might be adjusted (eg end set to today)
    pub start: Date,
    pub end: Date,
}

impl WorkingEntity {
    ///
    pub fn from(
        entity: Entity,
        colours: TimelineColours,
        measured_layout_params: MeasuredLayoutParams,
        zoomed_layout_params: ScalableLayoutParams,
        text_width: f64,
    ) -> Self {
        let row_height_with_padding =
            measured_layout_params.row_height_no_padding + (2.0 * zoomed_layout_params.padding_y);

        // Colours
        let entity_tag_colours = Colours::tag_colours();
        let (text_box, date_box) = if let Some(colour) = entity_tag_colours.entity_colours(&entity)
        {
            let text_box = colour;
            let date_box = Colour::lightened_colour(colour);
            (text_box, date_box)
        } else {
            let text_box = colours.entity.text_box.fill_colour;
            let date_box = colours.entity.date_box.fill_colour;
            (text_box, date_box)
        };
        let (text_box, date_box) = if true {
            let text_box = Colour::nearby_colour(text_box, 5);
            let date_box = Colour::nearby_colour(date_box, 5);
            (text_box, date_box)
        } else {
            (text_box, date_box)
        };

        // Text
        let text = TextWorking::from(
            entity.name().to_string(),
            text_width,
            zoomed_layout_params.font_size_px,
            colours.entity.text_colour,
        );

        // Text box (position calculated elsewhere)
        let text_box = FilledBox {
            position_and_size: PositionAndSize {
                position: Point { x: 0.0, y: 0.0 },
                width: text_width + (2.0 * zoomed_layout_params.padding_x),
                height: row_height_with_padding,
            },
            fill_colour: text_box,
            border_style: colours.entity.text_box.border,
        };

        // Date box (position and width calculated elsewhere)
        let date_box = FilledBox {
            position_and_size: PositionAndSize {
                position: Point { x: 0.0, y: 0.0 },
                width: 0.0,
                height: row_height_with_padding,
            },
            fill_colour: date_box,
            border_style: colours.entity.date_box.border,
        };

        // Start and end dates
        let start = entity.start();
        let end = entity.end().unwrap_or(Date::today());

        Self {
            entity,
            text,
            text_box,
            date_box,
            is_hovered_over: false,
            is_hightlighted: false,
            is_selected: false,
            is_filtered_out_by_date_range: false,
            is_filtered_out_by_bool_expr: false,
            row: 0,
            start,
            end,
        }
    }

    ///
    pub fn row(&self) -> usize {
        self.row
    }

    ///
    pub fn set_row(&mut self, row: usize) {
        self.row = row
    }

    // TODO: rename
    ///
    pub fn update_if_appropriate(
        &mut self,
        colours: TimelineColours,
        measured_layout_params: MeasuredLayoutParams,
        zoomed_layout_params: ScalableLayoutParams,
        text_width: f64,
    ) {
        let row_height_with_padding =
            measured_layout_params.row_height_no_padding + (2.0 * zoomed_layout_params.padding_y);

        // Text
        self.text.width = text_width;
        self.text.font_size = zoomed_layout_params.font_size_px;

        // Text box (position calculated elsewhere)
        self.text_box.position_and_size.width = text_width + (2.0 * zoomed_layout_params.padding_x);
        self.text_box.position_and_size.height = row_height_with_padding;
        self.text_box.border_style = colours.entity.text_box.border;

        // Date box (position and width calculated elsewhere)
        self.date_box.position_and_size.height = row_height_with_padding;
        self.date_box.border_style = colours.entity.date_box.border;
    }

    ///
    pub fn _update_fill_colours(&mut self, colours: TimelineColours) {
        self.text_box.fill_colour = Colour::nearby_colour(colours.entity.text_box.fill_colour, 5);
        self.date_box.fill_colour = Colour::nearby_colour(colours.entity.date_box.fill_colour, 5);
    }

    ///
    pub fn is_filtered_out(&self) -> bool {
        self.is_filtered_out_by_bool_expr || self.is_filtered_out_by_date_range
    }

    /// Calculate the entity's minimum x position/value
    pub fn min_x(&self) -> f64 {
        self.text_box.position_and_size.position.x
    }

    /// Calculate the entity's maximum x position/value
    pub fn max_x(&self) -> f64 {
        self.text_box
            .position_and_size
            .max_x()
            .max(self.date_box.position_and_size.max_x())
    }

    /// Calculate the entity's maximum y position/value
    pub fn max_y(&self) -> f64 {
        self.text_box
            .position_and_size
            .max_y()
            .max(self.date_box.position_and_size.max_y())
    }

    /// Clone the entity and add an offset.  Used when moving the timeline so
    /// that nothing else needs to be re-calculated
    pub fn clone_with_added_offset(&mut self, x: f64, y: f64) -> Self {
        let mut entity = self.clone();
        entity.text.add_offset(x, y);
        entity.text_box.position_and_size.add_offset(x, y);
        entity.date_box.position_and_size.add_offset(x, y);
        entity
    }

    // Offset the text inside the box so that it sticks to the left of the
    // screen while there is space left to do so
    pub(crate) fn adjust_sticky_text(&mut self, padding_x: f64) {
        let text_width = self.text.width;
        let text_box_width = self.text_box.position_and_size.width;
        let date_box_width = self.date_box.position_and_size.width;
        let box_width = text_box_width.max(date_box_width);
        let box_free_space = box_width - text_width - (2.0 * padding_x);
        if self.text.top_left.x < padding_x {
            if self.text.top_left.x < -(box_free_space - padding_x) {
                self.text.top_left.x += box_free_space;
            } else {
                self.text.top_left.x = padding_x;
            }
        }
    }

    ///
    pub(crate) fn update_filtered_by_date_range(&mut self, date_range: &TimelineDateRange) {
        if let Some(start_date_cutoff) = date_range.start_date_cutoff {
            if self.entity.start() < start_date_cutoff {
                self.is_filtered_out_by_date_range = true;
                return;
            }
        }
        if let Some(end_date_cutoff) = date_range.end_date_cutoff {
            if let Some(entity_end_date) = self.entity.end() {
                if entity_end_date > end_date_cutoff {
                    self.is_filtered_out_by_date_range = true;
                    return;
                }
            } else if self.entity.start() > end_date_cutoff {
                self.is_filtered_out_by_date_range = true;
                return;
            }
        }
        self.is_filtered_out_by_date_range = false;
    }

    ///
    pub(crate) fn update_filtered_by_bool_tag_expr(&mut self, expr: &Option<BoolTagExpr>) {
        self.is_filtered_out_by_bool_expr = expr
            .as_ref()
            .map_or(false, |expr| !self.entity.matches_bool_tag_expr(expr));
    }
}
