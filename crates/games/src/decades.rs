// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Put entities into the correct decade
//!

use crate::{Answer, AnswerOption, GameError, GameManagement, Html, Stats, shuffle_answers};
use open_timeline_core::{Entity, HasIdAndName};
use rand::{Rng, seq::SliceRandom, thread_rng};
use std::collections::BTreeSet;

type Decade = i32;

/// The game variants
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum GameVariant {
    #[default]
    DecadeOfStart,
    DecadeOfEnd,
}

/// State for the "decades" game
#[derive(Debug, Default)]
pub struct DecadesGame {
    entity_pool: Vec<Entity>,
    pub stats: Stats,
    pub current_question: Option<Entity>,
    pub current_selection: Option<Decade>,
    pub correct_answer: Option<Decade>,
    pub current_options: Option<Vec<AnswerOption<Decade>>>,
    pub last_answer: Option<Answer>,
    pub game_variant: GameVariant,
}

struct Question {
    entity: Entity,
    options: Vec<AnswerOption<Decade>>,
}

impl DecadesGame {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_entity_pool(&mut self, entity_pool: Vec<Entity>) {
        self.entity_pool = entity_pool
    }

    pub fn generate_html_quiz(
        &mut self,
        question_count: usize,
    ) -> Result<(Vec<Html>, Vec<Html>), ()> {
        // Get Qs
        let mut questions = Vec::new();
        let mut rng = rand::thread_rng();
        loop {
            let entity = self.entity_pool.partial_shuffle(&mut rng, 1).0[0].clone();
            if let Ok(question) = generate_text_question(entity) {
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
            vec!["", "Questions", "", "", ""],
        ));
        html_answers.push(Html::html_opening_quiz_doc(
            "Quiz Answers",
            vec!["", "Questions", "", "", ""],
        ));

        // Create HTML tables for Qs and As
        for (i, question) in questions.iter().enumerate() {
            html_quiz.push(Html::quiz_table_row(vec![
                &i.to_string(),
                question.entity.name().as_str(),
                question.options[0]
                    .to_html_question(|date| date.to_string())
                    .str(),
                question.options[1]
                    .to_html_question(|date| date.to_string())
                    .str(),
                question.options[2]
                    .to_html_question(|date| date.to_string())
                    .str(),
            ]));

            html_answers.push(Html::quiz_table_row(vec![
                &i.to_string(),
                question.entity.name().as_str(),
                question.options[0]
                    .to_html_answer(|date| date.to_string())
                    .str(),
                question.options[1]
                    .to_html_answer(|date| date.to_string())
                    .str(),
                question.options[2]
                    .to_html_answer(|date| date.to_string())
                    .str(),
            ]));
        }

        // Finish HTML docs
        html_quiz.push(Html::quiz_html_doc_finish());
        html_answers.push(Html::quiz_html_doc_finish());

        // Return the HTML
        Ok((html_quiz, html_answers))
    }
}

impl GameManagement<Decade> for DecadesGame {
    fn new_game(&mut self) {
        self.entity_pool.clear();
        self.stats.reset();
        self.current_question = None;
        self.current_selection = None;
        self.correct_answer = None;
        self.current_options = None;
    }

    fn check_answer(&mut self, choice: Decade) -> Result<(), GameError> {
        let Some(correct) = self.correct_answer else {
            return Err(GameError::NoCorrectAnswer);
        };
        if correct == choice {
            self.stats.correct_round_count += 1;
            self.last_answer = Some(Answer::Correct);
            Ok(())
        } else {
            self.stats.incorrect_round_count += 1;
            self.last_answer = Some(Answer::Incorrect);
            Ok(())
        }
    }

    fn setup_next_round(&mut self) -> Result<(), GameError> {
        self.current_question = self.entity_pool.pop();
        let Some(entity) = self.current_question.as_ref() else {
            return Err(GameError::GeneratingQuestion);
        };
        self.stats.round += 1;
        let correct = start_decade_for_entity(entity.clone());
        let answers = generate_answer_options(correct);
        self.correct_answer = Some(correct);
        self.current_options = Some(answers);
        Ok(())
    }

    fn description(&mut self) -> String {
        String::from("Put entities into the correct decade")
    }
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

    loop {
        // Generate number of decades the incorrect decades are off by
        let distance = 10 * thread_rng().gen_range(1..=5) * thread_rng().gen_range(1..=5);

        // Create the first incorrect decade
        let incorrect_decade = {
            if thread_rng().gen_ratio(1, 2) {
                correct_decade + distance
            } else {
                correct_decade - distance
            }
        };

        incorrect_decades.insert(incorrect_decade);
        if incorrect_decades.len() == count {
            break;
        }
    }

    // Return incorrect decades
    incorrect_decades.into_iter().collect()
}
