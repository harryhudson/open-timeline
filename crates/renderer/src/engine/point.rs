// SPDX-License-Identifier: MIT

//!
//! Layout params
//!

use serde::Serialize;
use std::fmt::Debug;

pub type TimelineOffset = Point;
pub type Size = Point;
pub type Position = Point;

#[derive(Debug, Default, Copy, Clone, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn min(self, other: Self) -> Self {
        Point {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Point {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }
}
