// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Enter the year/decade in which the entity started/ended
//!

use crate::{Answer, GameError, GameManagement, Stats};
use open_timeline_core::{Date, Entity};
use rand::prelude::SliceRandom;

// TODO: also, not just exact year, but also decade option
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum GameVariant {
    #[default]
    StartDate,
    EndDate,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum YearOrDecade {
    #[default]
    Year,
    Decade,
}

/// State for the "which date" game
#[derive(Debug, Default)]
pub struct WhichDateGame {
    entity_pool: Vec<Entity>,
    pub variant: GameVariant,
    pub year_or_decade: YearOrDecade,
    pub stats: Stats,
    pub current_question: Option<Entity>,
    pub current_selection: Option<Date>,
    pub correct_answer: Option<i32>,
    pub last_answer: Option<Answer>,
}

impl WhichDateGame {
    /// Create new WhichDateGame
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_entity_pool(&mut self, entity_pool: Vec<Entity>) {
        self.entity_pool = entity_pool;
    }

    fn update_correct_answer(&mut self) {
        let correct_date = match &self.current_question {
            Some(entity) => match self.variant {
                GameVariant::StartDate => entity.start_year(),
                GameVariant::EndDate => todo!(),
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

    fn description(&mut self) -> String {
        let start_end = match self.variant {
            GameVariant::StartDate => "start",
            GameVariant::EndDate => "end",
        };
        let year_decade = match self.year_or_decade {
            YearOrDecade::Year => "year",
            YearOrDecade::Decade => "decade",
        };
        format!("What is the {start_end} {year_decade}?")
    }
}
