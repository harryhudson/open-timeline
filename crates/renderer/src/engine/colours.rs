// SPDX-License-Identifier: MIT

//!
//! Colours
//!

use serde::{Deserialize, Serialize};

use crate::colour::Colour;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimelineEntityColourModifier {
    Lighten,
    // Darken,
    // Colour(Colour),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineStyle {
    pub colour: Colour,
    pub thickness: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoxStyle {
    pub fill_colour: Colour,
    pub border: Option<LineStyle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HeadingStyle {
    pub rect: BoxStyle,
    pub text_colour: Colour,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EntityStyle {
    pub text_box: BoxStyle,
    pub date_box: BoxStyle,
    pub text_colour: Colour,
    pub click_colour: TimelineEntityColourModifier,
    pub hover_colour: TimelineEntityColourModifier,
}

// TODO: allow for more variation & options (eg a vec of colours)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackgroundColours {
    pub a: Colour,
    pub b: Colour,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TimelineColours {
    pub background: BackgroundColours,
    pub dividing_line: LineStyle,
    pub entity: EntityStyle,
    pub heading: HeadingStyle,
}

impl Default for TimelineColours {
    fn default() -> Self {
        Self {
            background: BackgroundColours {
                a: Colour::from_hex("#ffffff").unwrap(),
                b: Colour::from_hex("#e8f8ff").unwrap(),
            },
            dividing_line: LineStyle {
                colour: Colour::from_rgb(0, 0, 0),
                thickness: 0.5,
            },
            entity: EntityStyle {
                text_box: BoxStyle {
                    fill_colour: Colour::from_hex("#e6e5ea").unwrap(),
                    border: None,
                },
                date_box: BoxStyle {
                    fill_colour: Colour::from_hex("#86D695").unwrap(),
                    border: None,
                },
                text_colour: Colour::from_rgb(0, 0, 0),
                click_colour: TimelineEntityColourModifier::Lighten,
                hover_colour: TimelineEntityColourModifier::Lighten,
            },
            heading: HeadingStyle {
                rect: BoxStyle {
                    fill_colour: Colour::from_hex("#0000aa").unwrap(),
                    border: None,
                },
                text_colour: Colour::from_hex("#ffffff").unwrap(),
            },
        }
    }
}
