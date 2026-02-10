// SPDX-License-Identifier: MIT

//!
//! Primitives
//!

use crate::{LineStyle, Point, PositionAndSize, colour::Colour};
use serde::Serialize;
use std::fmt::Debug;

/// Information needed to draw text
#[derive(Debug, Clone, Serialize)]
pub struct TextOut {
    pub top_left: Point,
    pub text: String,
    pub colour: Colour,
    pub font_size: f64,
}

/// Information needed when working with text calculations
#[derive(Debug, Clone, Serialize)]
pub(crate) struct TextWorking {
    pub top_left: Point,
    pub text: String,
    pub width: f64,
    pub colour: Colour,
    pub font_size: f64,
}

impl TextWorking {
    pub fn from(text: String, width: f64, font_size: f64, colour: Colour) -> Self {
        Self {
            top_left: Point::default(),
            text,
            width,
            colour,
            font_size,
        }
    }

    pub fn add_offset(&mut self, x_offset: f64, y_offset: f64) {
        self.top_left.x += x_offset;
        self.top_left.y += y_offset;
    }
}

// TODO: make Border type, and make it an Option<Border> here
/// Information needed to draw a filled box
#[derive(Debug, Clone, Copy, Serialize)]
pub struct FilledBox {
    pub position_and_size: PositionAndSize,
    pub fill_colour: Colour,
    pub border_style: Option<LineStyle>,
}

// TODO: is the same as Background
/// Information needed to draw the timeline's deliminating lines
#[derive(Debug, Clone, Serialize)]
pub struct VerticalLine {
    pub x: f64,
    pub style: LineStyle,
}

/// Information needed to draw the timeline's backgrounds
#[derive(Debug, Clone, Serialize)]
pub struct Background {
    pub x: f64,
    pub width: f64,
    pub colour: Colour,
}
