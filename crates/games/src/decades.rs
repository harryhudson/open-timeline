// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Put entities into the correct decade
//!

mod game;
mod html;
mod wasm;

use crate::{Answer, AnswerOption, Stats, shuffle_answers};
use open_timeline_core::Entity;
use rand::{Rng, thread_rng};
use std::collections::BTreeSet;
use wasm_bindgen::prelude::wasm_bindgen;

type Decade = i32;

/// The game variants
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DecadesGameVariant {
    #[default]
    DecadeOfStart,
    DecadeOfEnd,
}

/// State for the "decades" game
#[derive(Debug, Default, Clone)]
pub struct DecadesGame {
    entity_pool: Vec<Entity>,
    stats: Stats,
    current_question: Option<Entity>,
    correct_answer: Option<Decade>,
    current_options: Option<Vec<AnswerOption<Decade>>>,
    last_answer: Option<Answer>,
    variant: DecadesGameVariant,
}

struct Question {
    entity: Entity,
    options: Vec<AnswerOption<Decade>>,
}

/// Generate a question
fn generate_text_question(_entity: Entity) -> Result<Question, ()> {
    todo!()
}

/// Generate answer choices using the correct decade
fn generate_answer_options(correct: Decade) -> Vec<AnswerOption<Decade>> {
    let incorrect = generate_incorrect_decades(2, correct);
    let mut answers = vec![AnswerOption::Correct(correct)];
    incorrect
        .into_iter()
        .for_each(|incorrect| answers.push(AnswerOption::Incorrect(incorrect)));
    shuffle_answers(&mut answers);
    answers
}

// TODO: add end year approach too
fn start_decade_for_entity(entity: Entity) -> Decade {
    (entity.start_year().value() / 10) * 10
}

/// Generate a number of incorrect decades using the correct decade supplied
fn generate_incorrect_decades(count: usize, correct_decade: Decade) -> Vec<Decade> {
    let mut incorrect_decades = BTreeSet::new();
    let range_upper = 8;
    let half_range = range_upper / 2;
    loop {
        // 0..=100
        let in_range = 10 * thread_rng().gen_range(0..=range_upper);

        // At most 50 above or below actual
        let incorrect_decade = correct_decade + in_range - (half_range * 10);

        // If not incorrect, retry
        if incorrect_decade == correct_decade {
            continue;
        }
        incorrect_decades.insert(incorrect_decade);
        if incorrect_decades.len() == count {
            break;
        }
    }
    incorrect_decades.into_iter().collect()
}
