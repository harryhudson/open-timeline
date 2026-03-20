// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Which started/ended first, left or right?
//!

mod game;
mod html;
mod wasm;

use crate::{Answer, GameError, GameManagement, Stats};
use log::info;
use open_timeline_core::Entity;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeftOrRight {
    Left,
    Right,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LeftRightGameVariant {
    #[default]
    FirstStarted,
    FirstEnded,
}

/// State for the "left right" game
#[derive(Debug, Default)]
pub struct LeftRightGame {
    entity_pool: Vec<Entity>,
    stats: Stats,
    current_question: Option<(Entity, Entity)>,
    correct_answer: Option<LeftOrRight>,
    last_answer: Option<Answer>,
    variant: LeftRightGameVariant,
}
