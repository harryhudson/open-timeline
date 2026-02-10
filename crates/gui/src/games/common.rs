// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Common game functionality for the egui frontend
//!

use crate::common::ToOpenTimelineType;
use crate::components::TimelineSubtimelineGui;
use crate::config::SharedConfig;
use crate::spawn_transaction_no_commit_send_result;
use eframe::egui::{Context, Ui};
use open_timeline_core::{IsReducedType, ReducedTimeline, TimelineView};
use open_timeline_crud::{CrudError, FetchById};
use open_timeline_games::Stats;
use open_timeline_gui_core::{Draw, Valid, ValidityAsynchronous};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    NotStarted,
    StartedWaitingForTimeline,
    WaitingForAnswer,
    WaitingForNextRound,
    Finished,
}

impl GameState {
    pub fn has_started(&self) -> bool {
        *self != GameState::NotStarted
    }
}

#[derive(Debug)]
pub struct GameTimelineSearchAndFetch {
    // TODO: correct?
    // TODO: remove `pub`
    ///
    pub timeline: Option<Result<TimelineView, CrudError>>,

    ///
    rx_timeline: Option<Receiver<Result<TimelineView, CrudError>>>,

    /// Timeline search bar
    timeline_search_bar: TimelineSubtimelineGui,

    /// The timeline the game is played with
    timeline_playing_with: Option<ReducedTimeline>,

    /// Database pool
    shared_config: SharedConfig,
}

impl GameTimelineSearchAndFetch {
    pub fn new(shared_config: SharedConfig) -> Self {
        Self {
            shared_config: Arc::clone(&shared_config),
            timeline: None,
            rx_timeline: None,
            timeline_search_bar: TimelineSubtimelineGui::new(
                shared_config,
                open_timeline_gui_core::ShowRemoveButton::No,
            ),
            timeline_playing_with: None,
        }
    }

    pub fn timeline_playing_with(&self) -> &Option<ReducedTimeline> {
        &self.timeline_playing_with
    }

    pub fn draw_timeline_search_bar(&mut self, ctx: &Context, ui: &mut Ui, state: GameState) {
        if state == GameState::NotStarted {
            ui.horizontal(|ui| {
                ui.label("Timeline");
                self.timeline_search_bar.draw(ctx, ui);
            });
            if self.timeline_search_bar.validity() == ValidityAsynchronous::Valid {
                let reduced_timeline = self.timeline_search_bar.to_opentimeline_type();
                self.timeline_playing_with = Some(reduced_timeline);
            } else {
                self.timeline_playing_with = None;
            }
        } else {
            ui.horizontal(|ui| {
                ui.label("Timeline");
                open_timeline_gui_core::Label::strong(
                    ui,
                    self.timeline_playing_with.as_ref().unwrap().name().as_str(),
                );
            });
        }
    }

    pub fn request_fetch_timeline(&mut self) {
        self.timeline = None;
        let shared_config = Arc::clone(&self.shared_config);
        let timeline_id = self.timeline_playing_with.as_ref().unwrap().id();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.rx_timeline = Some(rx);
        spawn_transaction_no_commit_send_result!(
            shared_config,
            bounded,
            tx,
            |transaction| async move { TimelineView::fetch_by_id(transaction, &timeline_id).await }
        );
    }

    pub fn check_for_fetch_response(&mut self) {
        if let Some(rx) = self.rx_timeline.as_mut() {
            if let Ok(result) = rx.try_recv() {
                self.rx_timeline = None;
                self.timeline = Some(result);
            }
        }
    }
}

/// Draw the game stats (e.g. number of correct & incorrect answers)
pub fn draw_stats(_ctx: &Context, ui: &mut Ui, stats: Stats) {
    ui.horizontal(|ui| {
        let percent_correct = if stats.round > 1 {
            format!("{:.0}%", stats.percent_correct())
        } else {
            String::from("N/A")
        };
        ui.horizontal(|ui| {
            open_timeline_gui_core::Label::strong(ui, "Round");
            ui.label(format!("{}", stats.round));
        });
        ui.separator();
        ui.horizontal(|ui| {
            open_timeline_gui_core::Label::strong(ui, "Correct");
            ui.label(format!("{}", stats.correct_round_count));
        });
        ui.separator();
        ui.horizontal(|ui| {
            open_timeline_gui_core::Label::strong(ui, "Incorrect");
            ui.label(format!("{}", stats.incorrect_round_count));
        });
        ui.separator();
        ui.horizontal(|ui| {
            open_timeline_gui_core::Label::strong(ui, "Correct (%)");
            ui.label(format!("{percent_correct}"));
        });
    });
}
