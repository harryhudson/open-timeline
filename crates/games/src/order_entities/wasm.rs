// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! WASM for the order entities game
//!

use super::*;
use crate::wasm::get_element;
use log::error;
use log::info;
use open_timeline_core::HasIdAndName;
use wasm_bindgen::JsCast;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlButtonElement, HtmlDivElement, HtmlInputElement, HtmlSpanElement, window};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmOrderEntitiesGame {
    inner: OrderEntitiesGame,
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
    submit_button: HtmlButtonElement,
}

#[wasm_bindgen]
impl WasmOrderEntitiesGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let document = window().unwrap().document().unwrap();
        let game = Self {
            inner: OrderEntitiesGame::new(),
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
            game_variant_input_start: get_element!(document, "input[start]", HtmlInputElement),
            game_variant_input_end: get_element!(document, "input[end]", HtmlInputElement),
            options_div: get_element!(document, "div[options]", HtmlDivElement),
            submit_button: get_element!(document, "button[submit]", HtmlButtonElement),
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

    pub fn check_answer(&mut self, choice: JsValue) -> Result<(), GameError> {
        let choice = serde_wasm_bindgen::from_value(choice).unwrap();
        self.inner.check_answer(choice)?;
        self.update_answer();
        Ok(())
    }

    pub fn set_variant(&mut self, variant: OrderEntitiesGameVariant) {
        self.inner.variant = variant;
        self.set_description()
    }

    pub fn new_game(&mut self) -> Result<Vec<HtmlDivElement>, GameError> {
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
        let rows = self.next_round()?;

        // Show game
        self.game_time_div.set_hidden(false);

        //
        Ok(rows)
    }

    pub fn next_round(&mut self) -> Result<Vec<HtmlDivElement>, GameError> {
        // Update whether buttons enabled
        self.next_round_button.set_disabled(true);
        self.end_game_button.set_disabled(true);
        self.submit_button.set_disabled(false);

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

        // Clear current options
        self.options_div.set_inner_html("");

        // Add new options
        let document = window().unwrap().document().unwrap();

        // We collect the rows so we can pass them back to JS (where event
        // handlers can be added easily)
        let mut rows = Vec::new();

        // Create the rows
        for entity in question {
            let div: HtmlDivElement = document.create_element("div").unwrap().dyn_into().unwrap();

            let up_button: HtmlButtonElement = document
                .create_element("button")
                .unwrap()
                .dyn_into()
                .unwrap();
            up_button.set_type("button");
            up_button.set_text_content(Some("▲"));
            up_button.set_attribute("up", "").unwrap();

            let down_button: HtmlButtonElement = document
                .create_element("button")
                .unwrap()
                .dyn_into()
                .unwrap();
            down_button.set_type("button");
            down_button.set_text_content(Some("▼"));
            down_button.set_attribute("down", "").unwrap();

            let text: HtmlSpanElement =
                document.create_element("span").unwrap().dyn_into().unwrap();
            text.set_text_content(Some(entity.name().as_str()));
            let json = serde_json::to_string(&entity).unwrap();
            text.dataset().set("open_timeline_entity", &json).unwrap();
            text.set_attribute("entity", "").unwrap();

            // Add buttons & text to div
            div.append_child(&up_button).unwrap();
            div.append_child(&down_button).unwrap();
            div.append_child(&text).unwrap();

            // Push onto the list of rows
            rows.push(div);
        }

        // Set status
        self.status_span.set_inner_text("None");

        // Return the rows
        Ok(rows)
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
        let document = window().unwrap().document().unwrap();
        for selector in ["button[up]", "button[down]"] {
            let buttons = document.query_selector_all(selector).unwrap();
            for i in 0..buttons.length() {
                let button = buttons
                    .get(i)
                    .unwrap()
                    .dyn_into::<HtmlButtonElement>()
                    .unwrap();
                button.set_disabled(true);
            }
        }

        // Get the last answer & update the status
        if let Some(answer) = self.inner.last_answer() {
            info!("Last answer: {answer:?}");
            match answer {
                Answer::Correct => self.status_span.set_inner_text("Correct"),
                Answer::Incorrect => self.status_span.set_inner_text("Incorrect"),
            }
        }

        // Update whether buttons enabled
        self.next_round_button.set_disabled(false);
        self.end_game_button.set_disabled(false);
        self.submit_button.set_disabled(true);
    }
}
