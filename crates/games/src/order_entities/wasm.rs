// SPDX-License-Identifier: GPL-3.0-or-later

//!
//!
//!

use super::*;
use crate::wasm::wasm_game_wrapper;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

wasm_game_wrapper!(WasmOrderEntitiesGame, OrderEntitiesGame);

#[wasm_bindgen]
impl WasmOrderEntitiesGame {
    pub fn set_entity_pool(&mut self, entities: JsValue) {
        let entities = serde_wasm_bindgen::from_value(entities).unwrap();
        self.inner.set_entity_pool(entities);
    }

    pub fn current_question(&self) -> JsValue {
        let current_question = self.inner.current_question();
        serde_wasm_bindgen::to_value(&current_question).unwrap()
    }

    pub fn check_answer(&mut self, choice: JsValue) -> Result<(), GameError> {
        let choice = serde_wasm_bindgen::from_value(choice).unwrap();
        self.inner.check_answer(choice)
    }

    pub fn set_variant(&mut self, variant: OrderEntitiesGameVariant) {
        self.inner.variant = variant;
    }
}
