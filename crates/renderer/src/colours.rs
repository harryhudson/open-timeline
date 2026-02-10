// SPDX-License-Identifier: MIT

//!
//! Generate, maniplate, and manage colours used when drawing a timeline
//!

use crate::colour::Colour;
use bool_tag_expr::{Tag, TagValue};
use open_timeline_core::{self, Entity};

// TODO: this could/should be a map Tag -> Colour, rather than String -> Colour
// NOTE: need to be able to set "white" but also "#ffaadd" (do in HTML frontend)
/// Set to highest up tag. if entity has 'battle' and 'professor' it gets the
/// battle colour (then exit loop immediately)
///
/// Note: no `#``
#[derive(Default)]
pub struct Colours(Vec<(Tag, Colour)>);

impl Colours {
    /// Create new Colours
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, tag: Tag, colour: Colour) {
        self.0.push((tag, colour));
    }

    /// Can leave null, and a preditable colour will be used (consistent,
    /// without having to do by hand or all be the same)
    pub fn tag_colours() -> Self {
        let mut colours = Self(Vec::new());
        colours.add(
            Tag::from(None, TagValue::from(&"person").unwrap()),
            Colour::from_any_string("person"),
        );
        colours.add(
            Tag::from(None, TagValue::from(&"battle").unwrap()),
            Colour::from_hex("#ff0000").unwrap(),
        );
        colours.add(
            Tag::from(None, TagValue::from(&"book").unwrap()),
            Colour::from_hex("#aa3034").unwrap(),
        );
        colours.add(
            Tag::from(None, TagValue::from(&"novel").unwrap()),
            Colour::from_any_string("novel"),
        );
        colours
    }

    /// Get the colour for the tag
    pub fn tag_colour(&self, tag: Tag) -> Option<Colour> {
        for known_tag in &self.0 {
            if known_tag.0 == tag {
                return Some(known_tag.1);
            }
        }
        None
    }

    /// Get the colour for an entity based on its tags
    pub fn entity_colours(&self, entity: &Entity) -> Option<Colour> {
        if let Some(tags) = entity.tags() {
            for known_tag in &self.0 {
                if tags.contains(&known_tag.0) {
                    return Some(known_tag.1);
                }
            }
        }
        None
    }

    /// To get RGB as, say, #0affc3 (for CSS)
    fn _rgb_to_hex(&self, r: u8, g: u8, b: u8) -> String {
        // {:02x} means print as hex, requesting 2 chars (pad left with "0" if only 1 char otherwise)
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}
