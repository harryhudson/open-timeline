// SPDX-License-Identifier: GPL-3.0-or-later

//!
//!
//!

use super::*;
use crate::wasm::wasm_game_wrapper;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

wasm_game_wrapper!(WasmWereTheyAliveWhenGame, WereTheyAliveWhenGame);

#[wasm_bindgen]
impl WasmWereTheyAliveWhenGame {
    pub fn set_entity_pool(&mut self, entities: JsValue) {
        let entities = serde_wasm_bindgen::from_value(entities).unwrap();
        self.inner.set_entity_pool(entities);
    }

    pub fn current_question(&self) -> JsValue {
        let current_question = self.inner.current_question();
        serde_wasm_bindgen::to_value(&current_question).unwrap()
    }

    pub fn check_answer(&mut self, choice: bool) -> Result<(), GameError> {
        self.inner.check_answer(choice)
    }
}
