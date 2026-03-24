// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Core engine for the decades game
//!

use super::*;
use crate::{Answer, GameError, GameManagement, Stats};
use open_timeline_core::Entity;
use rand::seq::SliceRandom;

impl DecadesGame {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_entity_pool(&mut self, entity_pool: Vec<Entity>) {
        self.entity_pool = entity_pool
    }

    pub fn current_question(&self) -> Option<Entity> {
        self.current_question.clone()
    }

    pub fn correct_answer(&self) -> Option<Decade> {
        self.correct_answer
    }

    pub fn current_options(&self) -> Option<Vec<AnswerOption<Decade>>> {
        self.current_options.clone()
    }

    pub fn game_variant(&self) -> DecadesGameVariant {
        self.variant
    }

    pub fn game_variant_mut(&mut self) -> &mut DecadesGameVariant {
        &mut self.variant
    }
}

impl GameManagement<Decade> for DecadesGame {
    fn new_game(&mut self) {
        self.entity_pool.clear();
        self.stats.reset();
        self.current_question = None;
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
        if self.entity_pool.is_empty() {
            self.current_question = None;
            return Err(GameError::PoolIsNotFullEnough);
        }
        let mut rng = rand::thread_rng();
        let entity = self.entity_pool.partial_shuffle(&mut rng, 1).0[0].clone();
        self.current_question = Some(entity.clone());
        self.stats.round += 1;
        let correct = start_decade_for_entity(entity.clone());
        let answers = generate_answer_options(correct);
        self.correct_answer = Some(correct);
        self.current_options = Some(answers);
        Ok(())
    }

    fn description(&self) -> String {
        match self.variant {
            DecadesGameVariant::DecadeOfStart => {
                String::from("Put entities into the correct start decade")
            }
            DecadesGameVariant::DecadeOfEnd => {
                String::from("Put entities into the correct end decade")
            }
        }
    }

    fn stats(&self) -> Stats {
        self.stats
    }

    fn last_answer(&self) -> Option<Answer> {
        self.last_answer
    }
}
