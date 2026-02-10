// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The "were they alive" game for egui
//!

use crate::config::SharedConfig;
use crate::games::{GameState, GameTimelineSearchAndFetch, draw_stats};
use bool_tag_expr::TagValue;
use eframe::egui::{self, Align, Context, Layout, TextWrapMode, Ui, Vec2};
use open_timeline_games::{GameManagement, were_they_alive_when::*};
use open_timeline_gui_core::{Draw, widget_x_spacing};

#[derive(Debug)]
pub struct WereTheyAliveWhenGameGui {
    /// The game engine
    game: WereTheyAliveWhenGame,

    /// The current state of the game
    state: GameState,

    /// Search and fetch the timeline used to play the game
    game_timeline_search_and_fetch: GameTimelineSearchAndFetch,
}

impl WereTheyAliveWhenGameGui {
    /// Create new WereTheyAliveWhenGameGui
    pub fn new(shared_config: SharedConfig) -> Self {
        Self {
            game: WereTheyAliveWhenGame::new(),
            state: GameState::NotStarted,
            game_timeline_search_and_fetch: GameTimelineSearchAndFetch::new(shared_config),
        }
    }

    fn draw_question(&mut self, _ctx: &Context, ui: &mut Ui, enabled: bool) {
        if let Some(question) = &self.game.current_question {
            open_timeline_gui_core::Label::sub_heading(ui, question.str());

            let spacing = widget_x_spacing(ui);
            let width = (ui.available_width() - spacing) / 2.0;
            let height = ui.available_height() / 3.0;
            let button_size = Vec2::new(width, height);

            ui.columns(2, |ui| {
                // Left
                ui[0].with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.scope(|ui| {
                        ui.set_max_height(height);
                        let button = egui::Button::new("Yes")
                            .min_size(button_size)
                            .wrap_mode(TextWrapMode::Wrap);
                        if ui.add_enabled(enabled, button).clicked() {
                            let _ = self.game.check_answer(true);
                            self.state = GameState::WaitingForNextRound;
                        }
                    });
                });

                // Right
                ui[1].with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.set_max_height(height);
                    let button = egui::Button::new("No")
                        .min_size(button_size)
                        .wrap_mode(TextWrapMode::Wrap);
                    if ui.add_enabled(enabled, button).clicked() {
                        let _ = self.game.check_answer(false);
                        self.state = GameState::WaitingForNextRound;
                    }
                });
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

impl Draw for WereTheyAliveWhenGameGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        // Description
        open_timeline_gui_core::Label::description(ui, &self.game.description());
        ui.separator();

        // Search
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
                            let (people, not_people): (Vec<_>, Vec<_>) = timeline
                                .entities()
                                .as_deref()
                                .unwrap_or(&[])
                                .iter()
                                .cloned()
                                .partition(|entity| {
                                    entity.tags().clone().map_or(false, |tags| {
                                        tags.iter().any(|tag| {
                                            tag.value == TagValue::from(&"person").unwrap()
                                        })
                                    })
                                });
                            self.game.set_people_entity_pool(people);
                            self.game.set_not_people_entity_pool(not_people);
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
