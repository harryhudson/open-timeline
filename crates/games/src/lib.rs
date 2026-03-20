// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! *Part of the wider OpenTimeline project*
//!
//! This library crate provides all underlying mechanics for OpenTimeline games.
//! It does not provide a front end - to use these in applications, the application
//! must provide the user interface.
//!
//! This crate makes use of the basic OpenTimeline `core` crate for primitive
//! types, and is itself used by the `gui` crate as well as the OpenTimeline
//! website.
//!

mod html;
mod wasm;

pub mod decades;
pub mod left_right;
pub mod order_entities;
pub mod were_they_alive_when;
pub mod which_date;

use open_timeline_core::Date;
use rand::{Rng, seq::SliceRandom, thread_rng};
use std::collections::HashSet;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::html::Html;

/// Indicates answer correctness
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Incorrect,
    Correct,
}

/// Implementing types are games that can be managed externally
pub trait GameManagement<T> {
    // TODO: can this be derived for all games? I think they're all the same
    /// Start a new game (i.e. play round 1).  Note that this clears the
    /// entities held
    fn new_game(&mut self);

    /// Setup the next round (i.e. play the next round)
    fn setup_next_round(&mut self) -> Result<(), GameError>;

    /// Update the game state, noting whether the supplied answer is correct
    fn check_answer(&mut self, choice: T) -> Result<(), GameError>;

    /// Get the game's description
    fn description(&self) -> String;

    /// Get the game's stats
    fn stats(&self) -> Stats;

    /// Get whether the last answer was correct or incorrect (if there was one)
    fn last_answer(&self) -> Option<Answer>;
}

/// Possible game management errors
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameError {
    NoCorrectAnswer,
    PoolIsNotFullEnough,
    GeneratingQuestion,
}

/// Game stats
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    ///
    #[wasm_bindgen(readonly)]
    pub round: i32,
    ///
    #[wasm_bindgen(readonly)]
    pub correct_round_count: i32,
    ///
    #[wasm_bindgen(readonly)]
    pub incorrect_round_count: i32,
}

impl Stats {
    /// Reset the game stats
    pub fn reset(&mut self) {
        self.round = 0;
        self.correct_round_count = 0;
        self.incorrect_round_count = 0;
    }

    /// Calculate the % of rounds/questions answered correctly
    pub fn percent_correct(&self) -> i32 {
        (100.0
            * (self.correct_round_count as f32
                / (self.incorrect_round_count + self.correct_round_count) as f32)) as i32
    }
}

// TODO: what is this for?
/// Possible game answer options.  Holds the thing in the variants.
#[derive(Clone, Copy, Debug)]
pub enum AnswerOption<T> {
    Correct(T),
    Incorrect(T),
}

impl<T> AnswerOption<T> {
    pub fn to_html_answer<F: Fn(&T) -> String>(&self, fn_to_get_str: F) -> Html {
        match self {
            Self::Correct(value) => Html(format!("<b>{}</b>", fn_to_get_str(value))),
            Self::Incorrect(value) => Html(fn_to_get_str(value)),
        }
    }

    pub fn to_html_question<F: Fn(&T) -> String>(&self, fn_to_get_str: F) -> Html {
        match self {
            Self::Correct(value) => Html(fn_to_get_str(value)),
            Self::Incorrect(value) => Html(fn_to_get_str(value)),
        }
    }
}

/// Generate the given number of incorrect dates using the supplied date
pub fn generate_incorrect_dates(count: usize, correct_date: Date) -> Vec<Date> {
    let mut incorrect_dates = HashSet::new();

    loop {
        // Generate number of decades the incorrect decades are off by
        let distance = thread_rng().gen_range(1..=10) * thread_rng().gen_range(1..=10);

        // Create the first incorrect decade
        let _incorrect_decade = {
            if thread_rng().gen_ratio(1, 2) {
                correct_date.year().value() + distance
            } else {
                correct_date.year().value() - distance
            }
        };

        // Create the first incorrect year
        let incorrect_year = {
            if thread_rng().gen_ratio(1, 2) {
                correct_date.year().value() + distance
            } else {
                correct_date.year().value() - distance
            }
        };

        // Create the incorrect date
        let incorrect_date = Date::from(None, None, incorrect_year.into()).unwrap();

        incorrect_dates.insert(incorrect_date);
        if incorrect_dates.len() == count {
            break;
        }
    }

    incorrect_dates.into_iter().collect()
}

/// Shuffle the answer options
pub fn shuffle_answers<T>(options: &mut [AnswerOption<T>]) {
    options.shuffle(&mut thread_rng())
}
