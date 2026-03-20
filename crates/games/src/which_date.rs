// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Enter the year/decade in which the entity started/ended
//!

mod game;
mod html;
mod wasm;

use crate::{Answer, GameError, GameManagement, Stats};
use open_timeline_core::Entity;
use rand::prelude::SliceRandom;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum WhichDateGameVariant {
    #[default]
    StartDate,
    EndDate,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum YearOrDecade {
    #[default]
    Year,
    Decade,
}

/// State for the "which date" game
#[derive(Debug, Default)]
pub struct WhichDateGame {
    entity_pool: Vec<Entity>,
    year_or_decade: YearOrDecade,
    stats: Stats,
    current_question: Option<Entity>,
    correct_answer: Option<i32>,
    last_answer: Option<Answer>,
    variant: WhichDateGameVariant,
}
