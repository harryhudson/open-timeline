// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The "left right" game for egui
//!

use crate::config::SharedConfig;
use crate::games::{GameState, GameTimelineSearchAndFetch, draw_stats};
use eframe::egui::{self, Align, Context, Layout, TextWrapMode, Ui, Vec2};
use open_timeline_core::HasIdAndName;
use open_timeline_games::GameManagement;
use open_timeline_games::left_right::{LeftOrRight, LeftRightGame};
use open_timeline_gui_core::{Draw, widget_x_spacing};

#[derive(Debug)]
pub struct LeftRightGameGui {
    /// The game engine
    game: LeftRightGame,

    /// The current state of the game
    state: GameState,

    /// Which (left or right) option was chosen last
    last_question_option_chosen: Option<LeftOrRight>,

    /// Search and fetch the timeline used to play the game
    game_timeline_search_and_fetch: GameTimelineSearchAndFetch,
}

impl LeftRightGameGui {
    /// Create new LeftRightGameGui
    pub fn new(shared_config: SharedConfig) -> Self {
        Self {
            game: LeftRightGame::new(),
            state: GameState::NotStarted,
            last_question_option_chosen: None,
            game_timeline_search_and_fetch: GameTimelineSearchAndFetch::new(shared_config),
        }
    }

    fn draw_question(&mut self, _ctx: &Context, ui: &mut Ui, enabled: bool) {
        if let Some((left, right)) = self.game.current_question.clone() {
            let spacing = widget_x_spacing(ui);
            let width = (ui.available_width() - spacing) / 2.0;
            let height = ui.available_height() / 3.0;
            let button_size = Vec2::new(width, height);
            ui.columns(2, |ui| {
                // Left
                ui[0].with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.scope(|ui| {
                        ui.set_max_height(height);
                        let button = egui::Button::new(left.name().as_str())
                            .min_size(button_size)
                            .wrap_mode(TextWrapMode::Wrap);

                        if ui.add_enabled(enabled, button).clicked() {
                            let _ = self.game.check_answer(LeftOrRight::Left);
                            self.last_question_option_chosen = Some(LeftOrRight::Left);
                            self.state = GameState::WaitingForNextRound;
                        }
                    });
                });

                // Right
                ui[1].with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.set_max_height(height);
                    let button = egui::Button::new(right.name().as_str())
                        .min_size(button_size)
                        .wrap_mode(TextWrapMode::Wrap);

                    if ui.add_enabled(enabled, button).clicked() {
                        let _ = self.game.check_answer(LeftOrRight::Right);
                        self.last_question_option_chosen = Some(LeftOrRight::Right);
                        self.state = GameState::WaitingForNextRound;
                    }
                });
            });
        } else {
            open_timeline_gui_core::Label::weak(ui, "No questions");
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

impl Draw for LeftRightGameGui {
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
