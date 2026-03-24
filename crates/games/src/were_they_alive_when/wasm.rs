// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! WASM for the were they alive when game
//!

use super::*;
use crate::wasm::get_element;
use log::error;
use log::info;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlButtonElement, HtmlDivElement, HtmlSpanElement, window};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmWereTheyAliveWhenGame {
    inner: WereTheyAliveWhenGame,
    entities: Vec<Entity>,

    // Common
    game_time_div: HtmlDivElement,
    description_span: HtmlSpanElement,
    status_span: HtmlSpanElement,
    stats_round_span: HtmlSpanElement,
    stats_correct_span: HtmlSpanElement,
    stats_incorrect_span: HtmlSpanElement,
    next_round_button: HtmlButtonElement,
    new_game_button: HtmlButtonElement,
    end_game_button: HtmlButtonElement,

    // Game-specific
    yes_button: HtmlButtonElement,
    no_button: HtmlButtonElement,
    question_span: HtmlSpanElement,
}

#[wasm_bindgen]
impl WasmWereTheyAliveWhenGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let document = window().unwrap().document().unwrap();
        let game = Self {
            inner: WereTheyAliveWhenGame::new(),
            entities: Vec::new(),

            // Common
            game_time_div: get_element!(document, "div[game-time]", HtmlDivElement),
            description_span: get_element!(document, "span[description]", HtmlSpanElement),
            status_span: get_element!(document, "span[status]", HtmlSpanElement),
            stats_round_span: get_element!(document, "span[round]", HtmlSpanElement),
            stats_correct_span: get_element!(document, "span[correct]", HtmlSpanElement),
            stats_incorrect_span: get_element!(document, "span[incorrect]", HtmlSpanElement),
            next_round_button: get_element!(document, "button[next-round]", HtmlButtonElement),
            new_game_button: get_element!(document, "button[new-game]", HtmlButtonElement),
            end_game_button: get_element!(document, "button[end-game]", HtmlButtonElement),

            // Game-specific
            yes_button: get_element!(document, "button[yes]", HtmlButtonElement),
            no_button: get_element!(document, "button[no]", HtmlButtonElement),
            question_span: get_element!(document, "span[question]", HtmlSpanElement),
        };
        game.set_description();
        game
    }

    pub fn set_entity_pool(&mut self, entities: JsValue) {
        let entities: Vec<Entity> = serde_wasm_bindgen::from_value(entities).unwrap();
        info!("setting entities: {entities:?}");
        self.entities = entities.clone();
        self.inner.set_entity_pool(entities);
    }

    pub fn current_question(&self) -> JsValue {
        let current_question = self.inner.current_question();
        serde_wasm_bindgen::to_value(&current_question).unwrap()
    }

    pub fn check_answer(&mut self, choice: bool) -> Result<(), GameError> {
        match choice {
            true => info!("Chose yes option"),
            false => info!("Chose no option"),
        }
        self.inner.check_answer(choice)?;
        self.update_answer();
        Ok(())
    }

    pub fn new_game(&mut self) -> Result<(), GameError> {
        // Start a new game
        info!("New game");
        self.inner.new_game();

        // Correct stats
        self.display_stats();

        // Use the entities to play the game
        info!("Setting entity pool (JS)");
        self.inner.set_entity_pool(self.entities.clone());

        // Disable buttons
        self.new_game_button.set_disabled(true);

        // Start
        self.next_round()?;

        // Show game
        self.game_time_div.set_hidden(false);

        //
        Ok(())
    }

    pub fn next_round(&mut self) -> Result<(), GameError> {
        // Disable the next round button
        self.next_round_button.set_disabled(true);
        self.end_game_button.set_disabled(true);

        // Enable the yes/no buttons
        self.yes_button.set_disabled(false);
        self.no_button.set_disabled(false);

        // Setup the next round
        info!("Setup next round");
        if let Err(error) = self.inner.setup_next_round() {
            error!("No next question");
            error!("Error: {error:?}");
            self.status_span
                .set_inner_text("No questions (choose another timeline)");
            return Err(error);
        }

        // Get the current question
        info!("Current question");
        let question = self.inner.current_question().unwrap();
        info!("{question:?}");

        // Use the current question to update the HTML
        self.question_span
            .set_inner_text(&format!("{}", question.text));

        // Set status
        self.status_span.set_inner_text("None");

        //
        Ok(())
    }

    pub fn end_game(&self) {
        // End game
        info!("End game");

        // Enable buttons
        self.new_game_button.set_disabled(false);

        // Disable end game button
        self.end_game_button.set_disabled(true);

        // Hide game
        self.game_time_div.set_hidden(true);

        // Update status
        self.status_span.set_inner_text("Ended game");
    }

    fn set_description(&self) {
        self.description_span
            .set_inner_text(&self.inner.description())
    }

    fn display_stats(&self) {
        let stats = self.inner.stats();
        self.stats_round_span
            .set_inner_text(&format!("{}", stats.round));
        self.stats_correct_span
            .set_inner_text(&format!("{}", stats.correct_round_count));
        self.stats_incorrect_span
            .set_inner_text(&format!("{}", stats.incorrect_round_count));
    }

    fn update_answer(&self) {
        // Update stats
        self.display_stats();

        // Disable the buttons
        self.yes_button.set_disabled(true);
        self.no_button.set_disabled(true);

        // Get the last answer & update the status
        if let Some(answer) = self.inner.last_answer() {
            info!("Last answer: {answer:?}");
            match answer {
                Answer::Correct => self.status_span.set_inner_text("Correct"),
                Answer::Incorrect => self.status_span.set_inner_text("Incorrect"),
            }
        }

        // Enable the next round & end game buttons
        self.next_round_button.set_disabled(false);
        self.end_game_button.set_disabled(false);
    }
}
