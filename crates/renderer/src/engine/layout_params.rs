// SPDX-License-Identifier: MIT

//!
//! Layout params
//!

use crate::Position;
use serde::Serialize;
use std::fmt::Debug;

/// Layout parameters that are derived from the size of measured text
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct MeasuredLayoutParams {
    pub year_width: f64,
    pub row_height_no_padding: f64,
}

/// Layout parameters that users can adjust and which are multiplied by the scale
#[derive(Debug, Clone, Copy)]
pub struct ScalableLayoutParams {
    pub row_margin: f64,
    pub min_inline_spacing: f64,
    pub padding_x: f64,
    pub padding_y: f64,
    pub font_size_px: f64,
    pub dividing_line_thickness: f64,
    pub entity_highlight_thickness: f64,
}

impl Default for ScalableLayoutParams {
    fn default() -> Self {
        ScalableLayoutParams {
            row_margin: 5.0,
            min_inline_spacing: 5.0,
            padding_x: 10.0,
            padding_y: 7.0,
            font_size_px: 12.0,
            dividing_line_thickness: 0.5,
            entity_highlight_thickness: 10.0,
        }
    }
}

/// A box that specifies the location and size of something (e.g. the location
/// and size of the text drawn for an entity)
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct PositionAndSize {
    /// The smallest x/y values (boxes grow down and to the right from here)
    pub position: Position,

    /// The width of the box (from which the largest x value can be derived)
    pub width: f64,

    /// The height of the box (from which the largest y value can be derived)
    pub height: f64,
}

impl PositionAndSize {
    pub fn add_offset(&mut self, x_offset: f64, y_offset: f64) {
        self.position.x += x_offset;
        self.position.y += y_offset;
    }

    /// Calculate the largest x value of the box
    pub fn max_x(&self) -> f64 {
        self.position.x + self.width
    }

    /// Calculate the largest y value of the box (i.e. how far the box grows
    /// downwards)
    pub fn max_y(&self) -> f64 {
        self.position.y + self.height
    }
}
