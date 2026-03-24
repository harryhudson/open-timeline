// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! WASM for the decades game
//!

use super::*;
use crate::wasm::get_element;
use crate::{GameError, GameManagement};
use log::error;
use log::info;
use open_timeline_core::HasIdAndName;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlButtonElement, HtmlDivElement, HtmlInputElement, HtmlSpanElement, window};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmDecadesGame {
    inner: DecadesGame,
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
    game_variant_input_start: HtmlInputElement,
    game_variant_input_end: HtmlInputElement,
    options_div: HtmlDivElement,
    question_span: HtmlSpanElement,
}

#[wasm_bindgen]
impl WasmDecadesGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let document = window().unwrap().document().unwrap();
        let game = Self {
            inner: DecadesGame::new(),
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
            options_div: get_element!(document, "div[options]", HtmlDivElement),
            game_variant_input_start: get_element!(document, "input[start]", HtmlInputElement),
            game_variant_input_end: get_element!(document, "input[end]", HtmlInputElement),
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

    pub fn check_answer(&mut self, choice: Decade) -> Result<(), GameError> {
        self.inner.check_answer(choice)?;
        self.update_answer();
        Ok(())
    }

    pub fn set_variant(&mut self, variant: DecadesGameVariant) {
        self.inner.variant = variant;
        self.set_description()
    }

    pub fn new_game(&mut self) -> Result<Vec<HtmlButtonElement>, GameError> {
        // Start a new game
        info!("New game");
        self.inner.new_game();

        // Correct stats
        self.display_stats();

        // Use the entities to play the game
        info!("Setting entity pool (JS)");
        self.inner.set_entity_pool(self.entities.clone());

        // Disable buttons
        self.game_variant_input_start.set_disabled(true);
        self.game_variant_input_end.set_disabled(true);
        self.new_game_button.set_disabled(true);

        // Start
        let buttons = self.next_round()?;

        // Show game
        self.game_time_div.set_hidden(false);

        //
        Ok(buttons)
    }

    pub fn next_round(&mut self) -> Result<Vec<HtmlButtonElement>, GameError> {
        self.next_round_button.set_disabled(true);
        self.end_game_button.set_disabled(true);

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

        // Set the HTML question
        self.question_span.set_inner_text(question.name().as_str());

        // Get current options
        let options = self.inner.current_options().unwrap();
        info!("options: {options:?}");

        // Clear current options
        self.options_div.set_inner_html("");

        // Add new options
        let document = window().unwrap().document().unwrap();

        // We collect the buttons so we can pass them back to JS (where event
        // handlers can be added easily)
        let mut buttons = Vec::new();

        // Create the buttons
        for option in options {
            let button: HtmlButtonElement = document
                .create_element("button")
                .unwrap()
                .dyn_into()
                .unwrap();

            button.set_type("button");
            button.set_disabled(false);
            button.set_attribute("option", "").unwrap();

            let (decade, attr) = match option {
                AnswerOption::Correct(decade) => (decade, "correct"),
                AnswerOption::Incorrect(decade) => (decade, "false"),
            };

            button.set_text_content(Some(&format!("{decade}")));
            button.set_attribute(attr, "").unwrap();

            // Push onto the list of buttons
            buttons.push(button);
        }

        // Set status
        self.status_span.set_inner_text("None");

        // Return the buttons
        Ok(buttons)
    }

    pub fn end_game(&self) {
        // End game
        info!("End game");

        // Enable buttons
        self.game_variant_input_start.set_disabled(false);
        self.game_variant_input_end.set_disabled(false);
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
            .set_inner_text(&self.inner.description());
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
        let document = window().unwrap().document().unwrap();
        let buttons = document.query_selector_all("button[option]").unwrap();
        for i in 0..buttons.length() {
            let button = buttons
                .get(i)
                .unwrap()
                .dyn_into::<HtmlButtonElement>()
                .unwrap();
            button.set_disabled(true);
        }

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
