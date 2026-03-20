// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! State whether the person was alive when some event happened/started/ended
//!
//! The answers are true/false.  The questions can be asked one at a time, or an
//! HTML page generated with a load of Qs, along with a seperate HTML page with
//! the answers so that they can be printed out (e.g. to give as homework)
//!

mod game;
mod html;
mod wasm;

use crate::{Answer, GameError, GameManagement, Stats};
use open_timeline_core::{Entity, HasIdAndName};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};

/// State for the "were they alive when" game
#[derive(Debug, Default)]
pub struct WereTheyAliveWhenGame {
    people_pool: Vec<Entity>,
    not_people_pool: Vec<Entity>,
    stats: Stats,
    current_question: Option<Question>,
    correct_answer: Option<bool>,
    last_answer: Option<Answer>,
}

/// A "were they alive when" question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    // TODO: Are these fields needed?
    person: Entity,
    not_person: Entity,
    answer: bool,
    text: String,
}

impl Question {
    pub fn str(&self) -> &str {
        &self.text
    }
}

// TODO: rename
fn generate_text_question(person: Entity, not_person: Entity) -> Result<Question, GameError> {
    match thread_rng().gen_ratio(1, 2) {
        true => generate_alive_when_start_question(person, not_person),
        false => {
            let end_question = generate_alive_when_end_question(person.clone(), not_person.clone());
            if end_question.is_err() {
                generate_alive_when_start_question(person, not_person)
            } else {
                end_question
            }
        }
    }
}

/// Generate a question using the end date of the entity that is a person
fn generate_alive_when_start_question(
    person: Entity,
    not_person: Entity,
) -> Result<Question, GameError> {
    let text = format!(
        "Was {} alive when {} started?",
        person.name(),
        not_person.name()
    );
    let mut answer = true;
    if person.start() > not_person.start() {
        answer = false;
    } else if let (Some(person_end), Some(not_person_end)) = (person.end(), not_person.end()) {
        if person_end < not_person_end {
            answer = false;
        }
    }
    Ok(Question {
        person,
        not_person,
        answer,
        text,
    })
}

/// Generate a question using the end date of the entity that isn't a person
fn generate_alive_when_end_question(
    person: Entity,
    not_person: Entity,
) -> Result<Question, GameError> {
    if not_person.end().is_none() {
        return Err(GameError::GeneratingQuestion);
    }
    let text = format!(
        "Was {} alive when {} ended?",
        person.name(),
        not_person.name()
    );
    let mut answer = true;
    if person.start() > not_person.end().unwrap() {
        answer = false;
    } else if let (Some(person_end), Some(not_person_end)) = (person.end(), not_person.end()) {
        if person_end < not_person_end {
            answer = false
        }
    }
    Ok(Question {
        person,
        not_person,
        answer,
        text,
    })
}
