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

pub mod decades;
pub mod left_right;
pub mod order_entities;
pub mod were_they_alive_when;
pub mod which_date;

use open_timeline_core::Date;
use rand::{Rng, seq::SliceRandom, thread_rng};
use std::collections::HashSet;

/// Indicates answer correctness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Correct,
    Incorrect,
}

/// Implementing types are games that can be managed externally
pub trait GameManagement<T> {
    // TODO: can this be derived for all games? I think they're all the same
    /// Start a new game (i.e. play round 1)
    fn new_game(&mut self);

    /// Setup the next round (i.e. play the next round)
    fn setup_next_round(&mut self) -> Result<(), GameError>;

    /// Update the game state, noting whether the supplied answer is correct
    fn check_answer(&mut self, choice: T) -> Result<(), GameError>;

    /// Get the game's description
    fn description(&mut self) -> String;
}

/// Possible game management errors
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameError {
    NoCorrectAnswer,
    PoolIsNotFullEnough,
    GeneratingQuestion,
}

/// Game stats
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub round: i32,
    pub correct_round_count: i32,
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

/// For HTML creation
pub struct Html(String);

// TODO: check column counts?
impl Html {
    /// Get the underlying `&str`
    pub fn str(&self) -> &str {
        &self.0
    }

    /// Get a single HTML string from a vector of HTML strings
    pub fn from_vec(html: Vec<Html>) -> Self {
        Html(
            html.into_iter()
                .map(|html| html.0)
                .collect::<Vec<String>>()
                .concat(),
        )
    }

    /// Begin HTML docs
    pub fn html_opening_quiz_doc(
        title: impl ToString,
        table_column_headings: Vec<impl ToString>,
    ) -> Self {
        let title = title.to_string();
        let table_column_headings: Vec<String> = table_column_headings
            .into_iter()
            .map(|heading| heading.to_string())
            .collect();
        let mut html = format!(
            r"
                <h1>{title}</h1>
                <table>
                    <tr>
                        <th></th>
                        <th>Question</th>
                        <th></th>
                        <th></th>
                        <th></th>
                    </tr>
            "
        );
        for heading in table_column_headings {
            html.push_str(&format!("<th>{heading}</th>"));
        }
        html.push_str("</tr>");
        Html(html)
    }

    pub fn quiz_table_row(table_column_content: Vec<impl ToString>) -> Self {
        let mut row = String::from("<tr>");
        for column in table_column_content {
            row.push_str(&format!("<td>{}</td>", column.to_string()));
        }
        row.push_str("</tr>");
        Html(row)
    }

    pub fn quiz_html_doc_finish() -> Self {
        Html(String::from("</table>"))
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
