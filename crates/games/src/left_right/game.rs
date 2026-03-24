// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Core engine for the left-right game
//!

use super::*;

impl LeftRightGame {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_entity_pool(&mut self, entity_pool: Vec<Entity>) {
        info!("Setting entity pool (len = {})", entity_pool.len());
        self.entity_pool = entity_pool;
    }

    pub fn current_question(&self) -> Option<(Entity, Entity)> {
        self.current_question.clone()
    }

    fn update_correct_answer(&mut self) {
        let (left, right) = match &self.current_question {
            Some((left, right)) => (left, right),
            None => {
                self.correct_answer = None;
                return;
            }
        };
        self.correct_answer = match self.variant {
            LeftRightGameVariant::FirstEnded => {
                if left.end().unwrap() < right.end().unwrap() {
                    Some(LeftOrRight::Left)
                } else {
                    Some(LeftOrRight::Right)
                }
            }
            LeftRightGameVariant::FirstStarted => {
                if left.start() < right.start() {
                    Some(LeftOrRight::Left)
                } else {
                    Some(LeftOrRight::Right)
                }
            }
        };
    }
}

impl GameManagement<LeftOrRight> for LeftRightGame {
    fn new_game(&mut self) {
        self.entity_pool.clear();
        self.stats.reset();
        self.current_question = None;
        self.correct_answer = None;
        self.last_answer = None;
    }

    fn check_answer(&mut self, choice: LeftOrRight) -> Result<(), GameError> {
        let correct_answer = self
            .correct_answer
            .clone()
            .ok_or(GameError::NoCorrectAnswer)?;
        if choice == correct_answer {
            self.stats.correct_round_count += 1;
            self.last_answer = Some(Answer::Correct);
        } else {
            self.stats.incorrect_round_count += 1;
            self.last_answer = Some(Answer::Incorrect);
        }
        Ok(())
    }

    // TODO: what if their dates are equal? Generate a new Q
    fn setup_next_round(&mut self) -> Result<(), GameError> {
        if self.entity_pool.len() < 2 {
            self.current_question = None;
            return Err(GameError::PoolIsNotFullEnough);
        }
        let mut rng = rand::thread_rng();
        let options = self.entity_pool.partial_shuffle(&mut rng, 2).0;
        self.current_question = Some((options[0].clone(), options[1].clone()));
        self.update_correct_answer();
        self.stats.round += 1;
        Ok(())
    }

    fn description(&self) -> String {
        match self.variant {
            LeftRightGameVariant::FirstStarted => {
                String::from("Which started first, left or right?")
            }
            LeftRightGameVariant::FirstEnded => String::from("Which ended first, left or right?"),
        }
    }

    fn stats(&self) -> Stats {
        self.stats
    }

    fn last_answer(&self) -> Option<Answer> {
        self.last_answer
    }
}
