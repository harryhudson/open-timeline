// SPDX-License-Identifier: GPL-3.0-or-later

//!
//!
//!

use super::*;
use bool_tag_expr::TagValue;

impl WereTheyAliveWhenGame {
    /// Create new WereTheyAliveWhenGame
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_entity_pool(&mut self, entity_pool: Vec<Entity>) {
        let (people, not_people): (Vec<_>, Vec<_>) = entity_pool.into_iter().partition(|entity| {
            entity.tags().clone().map_or(false, |tags| {
                tags.iter()
                    .any(|tag| tag.value == TagValue::from("person").unwrap())
            })
        });
        self.people_pool = people;
        self.not_people_pool = not_people;
    }

    pub fn current_question(&self) -> Option<Question> {
        self.current_question.clone()
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

    fn description(&self) -> String {
        String::from("State whether the person was alive when some event happened/started/ended")
    }

    fn stats(&self) -> Stats {
        self.stats
    }

    fn last_answer(&self) -> Option<Answer> {
        self.last_answer
    }
}
