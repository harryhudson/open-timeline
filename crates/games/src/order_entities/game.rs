// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Core engine for the order entities game
//!

use super::*;

impl OrderEntitiesGame {
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

    pub fn current_question(&self) -> Option<Vec<Entity>> {
        self.current_question.clone()
    }

    pub fn variant(&self) -> OrderEntitiesGameVariant {
        self.variant
    }

    pub fn variant_mut(&mut self) -> &mut OrderEntitiesGameVariant {
        &mut self.variant
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
            OrderEntitiesGameVariant::OrderByFirstEnded => {
                next_q_entities.sort_by(|a, b| {
                    a.end()
                        .partial_cmp(&b.end())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            OrderEntitiesGameVariant::OrderByFirstStarted => {
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

    fn description(&self) -> String {
        match self.variant {
            OrderEntitiesGameVariant::OrderByFirstStarted => {
                String::from("Order the entities by their start date (earliest at the top)")
            }
            OrderEntitiesGameVariant::OrderByFirstEnded => {
                String::from("Order the entities by their end date (earliest at the top)")
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
