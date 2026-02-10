// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! *Part of the wider OpenTimeline project*
//!
//! This library crate provides the GUI parts of the GUI application.  It is
//! used to build the OpenTimeline native GUI application.
//!

mod app;
mod app_colours;
mod common;
mod components;
mod config;
mod consts;
mod games;
mod macros;
mod primary_window;
mod shortcuts;
mod windows;

pub use app::OpenTimelineApp;
pub use config::Config;
pub use consts::DEFAULT_WINDOW_SIZES;

#[macro_use]
extern crate log;
