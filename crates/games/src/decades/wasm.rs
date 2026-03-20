// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! WASM
//!

use super::*;
use crate::wasm::wasm_game_wrapper;
use crate::{GameError, GameManagement};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

wasm_game_wrapper!(WasmDecadesGame, DecadesGame);

/// For passing options back to WASM (generics don't work with wasm_bindgen)
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WasmDecadesGameOption {
    pub answer: Answer,
    pub decade: Decade,
}

impl From<AnswerOption<Decade>> for WasmDecadesGameOption {
    fn from(value: AnswerOption<Decade>) -> Self {
        match value {
            AnswerOption::Correct(decade) => WasmDecadesGameOption {
                answer: Answer::Correct,
                decade,
            },
            AnswerOption::Incorrect(decade) => WasmDecadesGameOption {
                answer: Answer::Incorrect,
                decade,
            },
        }
    }
}

#[wasm_bindgen]
impl WasmDecadesGame {
    pub fn set_entity_pool(&mut self, entities: JsValue) {
        let entities = serde_wasm_bindgen::from_value(entities).unwrap();
        self.inner.set_entity_pool(entities);
    }

    pub fn current_question(&self) -> JsValue {
        let current_question = self.inner.current_question();
        serde_wasm_bindgen::to_value(&current_question).unwrap()
    }

    pub fn check_answer(&mut self, choice: Decade) -> Result<(), GameError> {
        self.inner.check_answer(choice)
    }

    pub fn current_options(&mut self) -> Option<Vec<WasmDecadesGameOption>> {
        self.inner
            .current_options()
            .map(|options| options.into_iter().map(|option| option.into()).collect())
    }

    pub fn set_variant(&mut self, variant: DecadesGameVariant) {
        self.inner.variant = variant;
    }
}
