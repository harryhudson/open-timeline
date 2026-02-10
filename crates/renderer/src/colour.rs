// SPDX-License-Identifier: MIT

//!
//! Generate, maniplate, and manage colours used when drawing a timeline
//!

use eframe::egui;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// The `Colour` type
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl From<Colour> for egui::Color32 {
    fn from(value: Colour) -> Self {
        egui::Color32::from_rgb(value.r, value.g, value.b)
    }
}

impl From<egui::Color32> for Colour {
    fn from(value: egui::Color32) -> Self {
        Colour::from_rgb(value.r(), value.g(), value.b())
    }
}

impl From<Colour> for [u8; 3] {
    fn from(value: Colour) -> Self {
        [value.r, value.g, value.b]
    }
}

impl From<[u8; 3]> for Colour {
    fn from(value: [u8; 3]) -> Self {
        Colour::from_rgb(value[0], value[1], value[2])
    }
}

impl Colour {
    /// Create a colour from RGB values
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Colour { r, g, b }
    }

    /// Create a colour from a hex colour (e.g. `#ab66ef`, `ab66ef`, `#ab66efff`).
    /// If the hex value has an alpha component, it is removed.
    pub fn from_hex<S: Into<String>>(hex_colour: S) -> Result<Self, ()> {
        let hex_colour = hex_colour.into();

        // TODO: improve this (removes the alpha part)
        let len = hex_colour.len();
        let hex_colour = if len == 8 || len == 9 {
            &hex_colour[0..(len - 2)]
        } else {
            &hex_colour
        };

        // Check the hex length
        let len = hex_colour.len();
        if len != 6 && len != 7 {
            return Err(());
        }

        // Get individual RGB hex digits
        // Work backwards so that it's independent of a leading "#"
        let r_hex = &hex_colour[(len - 6)..(len - 4)];
        let g_hex = &hex_colour[(len - 4)..(len - 2)];
        let b_hex = &hex_colour[(len - 2)..(len)];

        // Convert RGB hex digits to u8s
        let r = u32::from_str_radix(r_hex, 16);
        let g = u32::from_str_radix(g_hex, 16);
        let b = u32::from_str_radix(b_hex, 16);

        // If all ok use Colour::from_rgb(), otherwise return an empty Err
        match (r, g, b) {
            (Ok(r), Ok(g), Ok(b)) => {
                let r = r.to_le_bytes()[0];
                let g = g.to_le_bytes()[0];
                let b = b.to_le_bytes()[0];
                Ok(Colour::from_rgb(r, g, b))
            }
            _ => Err(()),
        }
    }

    /// Create a random colour
    pub fn random() -> Self {
        let (r, g, b) = rand::random::<(u8, u8, u8)>();
        Colour::from_rgb(r, g, b)
    }

    // TODO: rename, or re-type? Perhaps from_hashable
    /// Create a colour from a name (string) in a repeatable way (naive hash)
    pub fn from_any_string<S: Into<String>>(name: S) -> Self {
        let bytes = name.into().into_bytes();
        let mut div_1: u16 = 0;
        let mut div_2: u16 = 0;
        let mut div_3: u16 = 0;
        for (i, byte) in bytes.iter().enumerate() {
            if i % 3 == 0 {
                div_1 += *byte as u16;
                div_1 %= 256;
            }
            if (i + 1) % 3 == 0 {
                div_2 += *byte as u16;
                div_2 %= 256;
            }
            if (i + 2) % 3 == 0 {
                div_3 += *byte as u16;
                div_3 %= 256;
            }
        }
        Colour::from_rgb(div_1 as u8, div_2 as u8, div_3 as u8)
    }

    /// Get a colour as RGB values
    pub fn as_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Get a lighter shade of the specified colour
    pub fn lightened_colour(colour: Colour) -> Colour {
        let old_r: f64 = colour.r.into();
        let old_g: f64 = colour.g.into();
        let old_b: f64 = colour.b.into();
        let new_r: f64 = (old_r + (0.5 * (255.0 - old_r))).round();
        let new_g: f64 = (old_g + (0.5 * (255.0 - old_g))).round();
        let new_b: f64 = (old_b + (0.5 * (255.0 - old_b))).round();
        Colour::from_rgb(new_r as u8, new_g as u8, new_b as u8)
    }

    /// Get a colour nearby to the specified one. `10` is a good value for
    /// `max_component_offset`.
    pub fn nearby_colour(mut colour: Colour, max_component_offset: i8) -> Colour {
        let max_component_offset = max_component_offset.abs();
        colour.r = random_offset_colour_component(colour.r, max_component_offset);
        colour.g = random_offset_colour_component(colour.g, max_component_offset);
        colour.b = random_offset_colour_component(colour.b, max_component_offset);
        colour
    }
}

// TODO: rename?
fn random_offset_colour_component(colour: u8, plus_minus: i8) -> u8 {
    let mut rng = rand::thread_rng();
    let offset = rng.gen_range(-plus_minus..=plus_minus);
    colour.saturating_add_signed(offset)
}
