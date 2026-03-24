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
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    log::info!("Start OpenTimeline games WASM");
    Ok(())
}

/// Macro helper to get an HTML element using a query selector
macro_rules! get_element {
    ($document:expr, $selector:expr, $ty:ty) => {{
        use wasm_bindgen::JsCast;

        $document
            .query_selector($selector)
            .expect("query_selector failed")
            .expect(&format!("element not found: {}", $selector))
            .dyn_into::<$ty>()
            .expect(&format!(
                "element is not of expected type: {}",
                stringify!($ty)
            ))
    }};
}

pub(crate) use get_element;

impl From<GameError> for JsValue {
    fn from(error: GameError) -> Self {
        JsValue::from_str(&format!("{error:?}"))
    }
}
