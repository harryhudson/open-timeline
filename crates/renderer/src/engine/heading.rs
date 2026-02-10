// SPDX-License-Identifier: MIT

//!
//! Headings
//!

use crate::{FilledBox, TextOut};
use serde::Serialize;
use std::fmt::Debug;

/// Information needed to draw the timeline's headings
#[derive(Debug, Clone, Serialize)]
pub struct Heading {
    pub text: TextOut,
    pub text_box: FilledBox,
}

impl Heading {
    /// Clone the heading and add an offset.  Used when moving the timeline so
    /// that nothing else needs to be re-calculated
    pub fn add_offset(&mut self, x_offset: f64) -> Self {
        let mut heading_with_offset = self.clone();
        heading_with_offset.text.top_left.x += x_offset;
        heading_with_offset.text_box.position_and_size.position.x += x_offset;
        heading_with_offset
    }
}
