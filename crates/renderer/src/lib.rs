// SPDX-License-Identifier: MIT

//!
//! *Part of the wider OpenTimeline project*
//!
//! This crate facilitates the drawing of timelines.  It can be compiled for
//! native use as well as to WASM for use in the browser (or any other
//! environment running WASM).
//!
//! The core of the crate is intended to be a platform independent an engine
//! responsible for:
//!
//! - Managing the entities that are to be drawn
//! - Managing constraints (such as entity filtering or date limit)
//! - Handling and emitting events
//! - Providing a simple API for frontends
//!
//! The rest of the crate holds code for various frontends.  There are currently
//! only 2, but the number will grow over time (e.g. SVG, OpenGL, and WebGL).
//! The 2 currently offered frontends are:
//!
//! - HTML Canvas for browser rendering
//! - `egui` for native desktop rendering
//!
//! ## Usage
//!
//! To use in a native `egui` desktop application the crate can simply be
//! included like any other crate.
//!
//! To use in a browser one can use the following to compile to WASM:
//!
//! ```sh
//! wasm-pack build --target web
//! ```
//!
//! One can then use the JavaScript module provided for easier interfacing.
//!

extern crate console_error_panic_hook;

pub mod colour;
pub mod colours;
pub mod engine;
pub mod frontends;

pub use colour::*;
pub use engine::*;
pub use frontends::html_canvas::OpenTimelineRendererHtmlCanvas;
