// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The OpenTimeline desktop app
//!

use eframe::egui::{IconData, ViewportBuilder};
use open_timeline_gui::{DEFAULT_WINDOW_SIZES, OpenTimelineApp};
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

#[macro_use]
extern crate log;
extern crate simplelog;

/// Entry point for the native GUI desktop application
fn main() -> Result<(), eframe::Error> {
    // Setup logging
    let config_log = ConfigBuilder::new()
        .add_filter_allow_str("open_timeline")
        .build();

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        config_log,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    // Create a new tokio runtime so that we can use `tokio::spawn` elsewhere
    // without requiring every function be `async` (waiting is not acceptable
    // for GUI rendering.
    let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Move the runtime into its own thread and don't let it finish/exit.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(std::time::Duration::MAX).await;
            }
        })
    });

    // Create the OpenTimeline application
    let open_timeline_app = OpenTimelineApp::new();

    // Setup the main window's default options
    let main_viewport_options = ViewportBuilder::default()
        .with_inner_size([
            DEFAULT_WINDOW_SIZES.main_window.width,
            DEFAULT_WINDOW_SIZES.main_window.height,
        ])
        .with_icon(load_icon());

    // Setup the eframe options for a native application
    let options = eframe::NativeOptions {
        viewport: main_viewport_options,
        ..Default::default()
    };

    info!("Launching application");

    // Run the application
    eframe::run_native(
        "OpenTimeline",
        options,
        Box::new(|_cc| Ok(Box::new(open_timeline_app))),
    )
}

/// Load the app icon embedded in the binary
fn load_icon() -> IconData {
    let bytes = include_bytes!("../../assets/icons/icon_512.png");
    let image = image::load_from_memory(bytes).unwrap().into_rgba8();
    let (width, height) = image.dimensions();
    IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}
