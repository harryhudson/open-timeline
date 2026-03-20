// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! WASM
//!

use crate::GameError;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

extern crate console_error_panic_hook;

/// Setup WASM logging & console log any panics
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // TODO: where to put this?
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    log::info!("Start OpenTimeline");
    Ok(())
}

macro_rules! wasm_game_wrapper {
    ($name:ident, $inner:ty) => {
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub struct $name {
            inner: $inner,
        }

        #[wasm_bindgen::prelude::wasm_bindgen]
        impl $name {
            #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
            pub fn new() -> Self {
                Self {
                    inner: <$inner>::new(),
                }
            }

            pub fn new_game(&mut self) {
                self.inner.new_game();
            }

            pub fn setup_next_round(&mut self) -> Result<(), wasm_bindgen::JsValue> {
                Ok(self.inner.setup_next_round()?)
            }

            pub fn description(&self) -> String {
                self.inner.description()
            }

            pub fn stats(&self) -> Stats {
                self.inner.stats()
            }

            pub fn last_answer(&self) -> Option<Answer> {
                self.inner.last_answer()
            }
        }
    };
}

pub(crate) use wasm_game_wrapper;

impl From<GameError> for JsValue {
    fn from(error: GameError) -> Self {
        JsValue::from_str(&format!("{error:?}"))
    }
}
