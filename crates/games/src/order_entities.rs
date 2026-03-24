// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Order entities by their start/end date
//!

mod game;
mod html;
mod wasm;

use crate::{Answer, GameError, GameManagement, Stats};
use open_timeline_core::Entity;
use rand::{Rng, seq::SliceRandom, thread_rng};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum OrderEntitiesGameVariant {
    #[default]
    OrderByFirstStarted,
    OrderByFirstEnded,
}

/// State for the order entities game
#[derive(Debug, Default, Clone)]
pub struct OrderEntitiesGame {
    entity_pool: Vec<Entity>,
    min_entities_per_round: usize,
    max_entities_per_round: usize,
    stats: Stats,
    current_question: Option<Vec<Entity>>,
    correct_answer: Option<Vec<Entity>>,
    last_answer: Option<Answer>,
    variant: OrderEntitiesGameVariant,
}
