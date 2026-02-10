// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The "which date" game for egui
//!

use crate::config::SharedConfig;
use crate::games::{GameState, GameTimelineSearchAndFetch, draw_stats};
use eframe::egui::{self, Context, FontId, RichText, TextEdit, Ui};
use open_timeline_core::HasIdAndName;
use open_timeline_games::GameManagement;
use open_timeline_games::which_date::{GameVariant, WhichDateGame, YearOrDecade};
use open_timeline_gui_core::Draw;

#[derive(Debug)]
pub struct WhichDateGameGui {
    /// The game engine
    game: WhichDateGame,

    /// The number as a string
    number_as_str: String,

    /// The current state of the game
    state: GameState,

    /// Search and fetch the timeline used to play the game
    game_timeline_search_and_fetch: GameTimelineSearchAndFetch,
}

impl WhichDateGameGui {
    /// Create new WhichDateGameGui
    pub fn new(shared_config: SharedConfig) -> Self {
        Self {
            game: WhichDateGame::new(),
            number_as_str: String::new(),
            state: GameState::NotStarted,
            game_timeline_search_and_fetch: GameTimelineSearchAndFetch::new(shared_config),
        }
    }

    fn draw_question(&mut self, _ctx: &Context, ui: &mut Ui, enabled: bool) {
        let current_question = self.game.current_question.clone();
        if let Some(question) = current_question {
            ui.add_enabled_ui(enabled, |ui| {
                // Question
                open_timeline_gui_core::Label::sub_heading(ui, question.name().as_str());

                // Answer input
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.number_as_str)
                            .font(FontId::proportional(18.0))
                            .desired_width(ui.available_width()),
                    );
                    if self.game.year_or_decade == YearOrDecade::Decade {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.add(egui::Label::new(
                            RichText::new("s").font(FontId::proportional(14.0)),
                        ));
                    }
                });

                // Submit answer
                if enabled {
                    let answer = self.number_as_str.parse::<i32>();
                    ui.add_enabled_ui(answer.is_ok(), |ui| {
                        if open_timeline_gui_core::Button::tall_full_width(ui, "Submit").clicked() {
                            if let Ok(answer) = answer {
                                let _ = self.game.check_answer(answer);
                                self.state = GameState::WaitingForNextRound;
                            }
                        }
                    });
                }
            });
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

impl Draw for WhichDateGameGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        // Description
        open_timeline_gui_core::Label::description(ui, &self.game.description());
        ui.separator();

        // Search
        // Timeline search bar/label
        self.game_timeline_search_and_fetch
            .draw_timeline_search_bar(ctx, ui, self.state);
        ui.separator();

        // Radio button controls
        ui.horizontal(|ui| {
            ui.add_enabled_ui(self.state == GameState::NotStarted, |ui| {
                ui.radio_value(
                    &mut self.game.variant,
                    GameVariant::StartDate,
                    "Enter start date",
                );
                ui.radio_value(
                    &mut self.game.variant,
                    GameVariant::EndDate,
                    "Enter end date",
                );
                ui.separator();
                ui.radio_value(
                    &mut self.game.year_or_decade,
                    YearOrDecade::Decade,
                    "Enter decade",
                );
                ui.radio_value(
                    &mut self.game.year_or_decade,
                    YearOrDecade::Year,
                    "Enter year",
                );
            });
        });
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
                    self.number_as_str.clear();
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
