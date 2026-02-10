// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Order entities by their start/end date
//!

use crate::{Answer, GameError, GameManagement, Stats};
use open_timeline_core::Entity;
use rand::{Rng, seq::SliceRandom, thread_rng};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum GameVariant {
    #[default]
    OrderByFirstStarted,
    OrderByFirstEnded,
}

/// State for the order entities game
#[derive(Debug, Default)]
pub struct OrderEntitiesGame {
    pub entity_pool: Vec<Entity>,
    pub stats: Stats,
    pub current_question: Option<Vec<Entity>>,
    correct_answer: Option<Vec<Entity>>,
    pub last_answer: Option<Answer>,
    pub min_entities_per_round: usize,
    pub max_entities_per_round: usize,
    pub variant: GameVariant,
}

impl OrderEntitiesGame {
    /// Create new OrderEntitiesGame
    pub fn new() -> Self {
        Self {
            min_entities_per_round: 4,
            max_entities_per_round: 15,
            ..Default::default()
        }
    }

    pub fn set_entity_pool(&mut self, entity_pool: Vec<Entity>) {
        self.entity_pool = entity_pool;
    }
}

impl GameManagement<Vec<Entity>> for OrderEntitiesGame {
    fn new_game(&mut self) {
        self.entity_pool.clear();
        self.stats.reset();
        self.current_question = None;
        self.correct_answer = None;
        self.last_answer = None;
    }

    fn check_answer(&mut self, choice: Vec<Entity>) -> Result<(), GameError> {
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

    fn setup_next_round(&mut self) -> Result<(), GameError> {
        if self.entity_pool.len() < self.max_entities_per_round {
            // TODO
        }
        let entity_count =
            rand::thread_rng().gen_range(self.min_entities_per_round..=self.max_entities_per_round);
        let mut rng = rand::thread_rng();
        let mut next_q_entities = self
            .entity_pool
            .partial_shuffle(&mut rng, entity_count)
            .0
            .to_vec();
        match self.variant {
            GameVariant::OrderByFirstEnded => {
                next_q_entities.sort_by(|a, b| {
                    a.end()
                        .partial_cmp(&b.end())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            GameVariant::OrderByFirstStarted => {
                next_q_entities.sort_by(|a, b| {
                    a.start()
                        .partial_cmp(&b.start())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
        self.correct_answer = Some(next_q_entities.clone());
        next_q_entities.shuffle(&mut thread_rng());
        self.current_question = Some(next_q_entities);
        self.stats.round += 1;
        Ok(())
    }

    fn description(&mut self) -> String {
        match self.variant {
            GameVariant::OrderByFirstStarted => {
                String::from("Order the entities by their start date (earliest at the top)")
            }
            GameVariant::OrderByFirstEnded => {
                String::from("Order the entities by their end date (earliest at the top)")
            }
        }
    }
}
