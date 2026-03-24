// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Core engine for the which date game
//!

use super::*;

impl WhichDateGame {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_entity_pool(&mut self, entity_pool: Vec<Entity>) {
        self.entity_pool = entity_pool;
    }

    pub fn current_question(&self) -> Option<Entity> {
        self.current_question.clone()
    }

    pub fn variant_mut(&mut self) -> &mut WhichDateGameVariant {
        &mut self.variant
    }

    pub fn year_or_decade(&mut self) -> YearOrDecade {
        self.year_or_decade
    }

    pub fn year_or_decade_mut(&mut self) -> &mut YearOrDecade {
        &mut self.year_or_decade
    }

    fn update_correct_answer(&mut self) {
        let correct_date = match &self.current_question {
            Some(entity) => match self.variant {
                WhichDateGameVariant::StartDate => entity.start_year(),
                WhichDateGameVariant::EndDate => todo!(),
            },
            None => {
                self.correct_answer = None;
                return;
            }
        };

        self.correct_answer = Some(match self.year_or_decade {
            YearOrDecade::Decade => (correct_date.value() / 10) * 10,
            YearOrDecade::Year => correct_date.value(),
        });
    }
}

impl GameManagement<i32> for WhichDateGame {
    fn new_game(&mut self) {
        self.entity_pool.clear();
        self.stats.reset();
        self.current_question = None;
        self.correct_answer = None;
        self.last_answer = None;
    }

    fn check_answer(&mut self, choice: i32) -> Result<(), GameError> {
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
        if self.entity_pool.is_empty() {
            self.current_question = None;
            return Err(GameError::PoolIsNotFullEnough);
        }
        let mut rng = rand::thread_rng();
        let options = self.entity_pool.partial_shuffle(&mut rng, 1).0;
        self.current_question = Some(options[0].clone());
        self.update_correct_answer();
        self.stats.round += 1;
        Ok(())
    }

    fn description(&self) -> String {
        let start_end = match self.variant {
            WhichDateGameVariant::StartDate => "start",
            WhichDateGameVariant::EndDate => "end",
        };
        let year_decade = match self.year_or_decade {
            YearOrDecade::Year => "year",
            YearOrDecade::Decade => "decade",
        };
        format!("What is the {start_end} {year_decade}?")
    }

    fn stats(&self) -> Stats {
        self.stats
    }

    fn last_answer(&self) -> Option<Answer> {
        self.last_answer
    }
}
