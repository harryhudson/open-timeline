// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The "decades" game for egui
//!

use crate::config::SharedConfig;
use crate::games::{GameState, GameTimelineSearchAndFetch, draw_stats};
use eframe::egui::{self, Context, Ui, Vec2};
use open_timeline_core::HasIdAndName;
use open_timeline_games::{AnswerOption, GameManagement, decades::DecadesGame};
use open_timeline_gui_core::{Draw, widget_x_spacing};

#[derive(Debug)]
pub struct DecadesGameGui {
    ///  The game engine
    game: DecadesGame,

    /// The current state of the game
    state: GameState,

    /// Search and fetch the timeline used to play the game
    game_timeline_search_and_fetch: GameTimelineSearchAndFetch,
}

impl DecadesGameGui {
    /// Create new DecadesGameGui
    pub fn new(shared_config: SharedConfig) -> Self {
        Self {
            game: DecadesGame::new(),
            state: GameState::NotStarted,
            game_timeline_search_and_fetch: GameTimelineSearchAndFetch::new(shared_config),
        }
    }

    fn draw_question(&mut self, _ctx: &Context, ui: &mut Ui, enabled: bool) {
        if let Some(entity) = self.game.current_question.clone() {
            if let Some(answers) = self.game.current_options.clone() {
                let option_count = answers.len();
                let spacing = widget_x_spacing(ui) * (option_count - 1) as f32;
                let width = (ui.available_width() - spacing) / option_count as f32;
                let height = ui.available_height() / 2.0;
                let button_size = Vec2::new(width, height);

                ui.add_enabled_ui(enabled, |ui| {
                    open_timeline_gui_core::Label::sub_heading(ui, entity.name().as_str());
                    ui.horizontal(|ui| {
                        for answer in answers {
                            match answer {
                                AnswerOption::Correct(answer) => {
                                    let answer_button = ui.add_sized(
                                        button_size,
                                        egui::Button::new(format!("{answer}")),
                                    );
                                    if answer_button.clicked() {
                                        println!("Correct");
                                        self.game.current_selection = Some(answer);
                                        let _ = self.game.check_answer(answer);
                                        self.state = GameState::WaitingForNextRound;
                                    }
                                }
                                AnswerOption::Incorrect(answer) => {
                                    let answer_button = ui.add_sized(
                                        button_size,
                                        egui::Button::new(format!("{answer}")),
                                    );
                                    if answer_button.clicked() {
                                        println!("Incorrect");
                                        self.game.current_selection = Some(answer);
                                        let _ = self.game.check_answer(answer);
                                        self.state = GameState::WaitingForNextRound;
                                    }
                                }
                            };
                        }
                    });
                });
            } else {
                open_timeline_gui_core::Label::weak(ui, "No options");
                self.draw_new_game_button(ui);
            }
        } else {
            open_timeline_gui_core::Label::weak(ui, "No question");
            self.draw_new_game_button(ui);
        }
    }

    fn draw_new_game_button(&mut self, ui: &mut Ui) {
        if open_timeline_gui_core::Button::tall_full_width(ui, "New Game").clicked() {
            self.game.new_game();
            self.state = GameState::NotStarted;
        }
    }
}

impl Draw for DecadesGameGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        // Description
        open_timeline_gui_core::Label::description(ui, &self.game.description());
        ui.separator();

        // Timeline search bar/label
        self.game_timeline_search_and_fetch
            .draw_timeline_search_bar(ctx, ui, self.state);
        ui.separator();

        // Stats
        if self.state.has_started() {
            draw_stats(ctx, ui, self.game.stats);
            ui.separator();
        }

        // Controls
        match self.state {
            GameState::NotStarted => {
                ui.add_enabled_ui(
                    self.game_timeline_search_and_fetch
                        .timeline_playing_with()
                        .is_some(),
                    |ui| {
                        if open_timeline_gui_core::Button::tall_full_width(ui, "Start").clicked() {
                            self.game.new_game();
                            self.game_timeline_search_and_fetch.request_fetch_timeline();
                            self.state = GameState::StartedWaitingForTimeline;
                        }
                    },
                );
            }
            GameState::StartedWaitingForTimeline => {
                self.game_timeline_search_and_fetch
                    .check_for_fetch_response();
                if let Some(result) = self.game_timeline_search_and_fetch.timeline.as_ref() {
                    match result {
                        Ok(timeline) => {
                            if let Some(entities) = timeline.entities() {
                                self.game.set_entity_pool(entities.clone());
                            }
                            self.state = GameState::WaitingForAnswer;
                            let _ = self.game.setup_next_round();
                        }
                        Err(error) => {
                            // TODO
                            panic!("{error}");
                        }
                    }
                }
            }
            GameState::WaitingForAnswer => {
                self.draw_question(ctx, ui, true);
            }
            GameState::WaitingForNextRound => {
                self.draw_question(ctx, ui, false);
                ui.separator();
                if let Some(last_answer) = self.game.last_answer.as_ref() {
                    ui.horizontal(|ui| {
                        ui.label("Last Answer");
                        open_timeline_gui_core::Label::strong(ui, &format!("{last_answer:?}"));
                    });
                    ui.separator();
                }
                if open_timeline_gui_core::Button::tall_full_width(ui, "End").clicked() {
                    self.state = GameState::Finished;
                }
                if open_timeline_gui_core::Button::tall_full_width(ui, "Next Round").clicked() {
                    let _ = self.game.setup_next_round();
                    self.state = GameState::WaitingForAnswer;
                }
            }
            GameState::Finished => {
                self.draw_new_game_button(ui);
            }
        }
    }
}
