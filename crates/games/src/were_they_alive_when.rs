// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! State whether the person was alive when some event happened/started/ended
//!
//! The answers are true/false.  The questions can be asked one at a time, or an
//! HTML page generated with a load of Qs, along with a seperate HTML page with
//! the answers so that they can be printed out (e.g. to give as homework)
//!

use crate::{Answer, GameError, GameManagement, Html, Stats};
use open_timeline_core::{Entity, HasIdAndName};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{Rng, thread_rng};

/// State for the "were they alive when" game
#[derive(Debug, Default)]
pub struct WereTheyAliveWhenGame {
    people_pool: Vec<Entity>,
    not_people_pool: Vec<Entity>,
    pub stats: Stats,
    pub current_question: Option<Question>,
    correct_answer: Option<bool>,
    pub last_answer: Option<Answer>,
}

/// A "were they alive when" question
#[derive(Debug)]
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

impl WereTheyAliveWhenGame {
    /// Create new WereTheyAliveWhenGame
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_people_entity_pool(&mut self, people_pool: Vec<Entity>) {
        self.people_pool = people_pool;
    }

    pub fn set_not_people_entity_pool(&mut self, not_people_pool: Vec<Entity>) {
        self.not_people_pool = not_people_pool;
    }

    pub fn generate_html_quiz(&mut self, question_count: usize) -> Result<(Html, Html), ()> {
        // Get Qs
        let mut questions = Vec::new();
        let mut rng = rand::thread_rng();
        loop {
            // TODO: bounds checking (is there a .get() or similar?)
            let person = self.people_pool.partial_shuffle(&mut rng, 1).0[0].clone();
            let not_person = self.not_people_pool.partial_shuffle(&mut rng, 1).0[0].clone();
            if let Ok(question) = generate_text_question(person, not_person) {
                questions.push(question);
            }
            if questions.len() == question_count {
                break;
            }
        }

        // Begin HTML docs
        let mut html_quiz = Vec::new();
        let mut html_answers = Vec::new();
        html_quiz.push(Html::html_opening_quiz_doc(
            "Quiz Questions",
            vec!["", "Question", "", ""],
        ));
        html_answers.push(Html::html_opening_quiz_doc(
            "Quiz Answers",
            vec!["", "Question", "Answer"],
        ));

        // Create HTML tables for Qs and As
        for (i, question) in questions.iter().enumerate() {
            html_quiz.push(Html::quiz_table_row(vec![
                &i.to_string(),
                &question.text,
                "T",
                "F",
            ]));

            html_answers.push(Html::quiz_table_row(vec![
                &i.to_string(),
                &question.text,
                &question.answer.to_string(),
            ]));
        }

        // Finish HTML docs
        html_quiz.push(Html::quiz_html_doc_finish());
        html_answers.push(Html::quiz_html_doc_finish());

        // Return the HTML
        Ok((Html::from_vec(html_quiz), Html::from_vec(html_answers)))
    }
}

impl GameManagement<bool> for WereTheyAliveWhenGame {
    fn new_game(&mut self) {
        self.people_pool.clear();
        self.not_people_pool.clear();
        self.stats.reset();
        self.current_question = None;
        self.correct_answer = None;
        self.last_answer = None;
    }

    fn check_answer(&mut self, choice: bool) -> Result<(), GameError> {
        let correct_answer = self.correct_answer.ok_or(GameError::NoCorrectAnswer)?;
        if choice == correct_answer {
            self.stats.correct_round_count += 1;
            self.last_answer = Some(Answer::Correct);
        } else {
            self.stats.incorrect_round_count += 1;
            self.last_answer = Some(Answer::Incorrect);
        }
        Ok(())
    }

    fn setup_next_round(&mut self) -> Result<(), GameError> {
        let person = self.people_pool.iter().choose(&mut thread_rng()).cloned();
        let not_person = self
            .not_people_pool
            .iter()
            .choose(&mut thread_rng())
            .cloned();
        let (person, not_person) = match (person, not_person) {
            (Some(person), Some(not_person)) => (person, not_person),
            _ => return Err(GameError::PoolIsNotFullEnough),
        };
        let question = generate_text_question(person, not_person)?;
        self.correct_answer = Some(question.answer);
        self.current_question = Some(question);
        self.stats.round += 1;
        Ok(())
    }

    fn description(&mut self) -> String {
        String::from("State whether the person was alive when some event happened/started/ended")
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
