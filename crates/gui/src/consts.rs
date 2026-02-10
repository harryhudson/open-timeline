// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Some configuration consts
//!

pub struct WindowSizes {
    pub main_window: WindowSize,
    pub entity_edit: WindowSize,
    pub entity_view: WindowSize,
    pub timeline_edit: WindowSize,
    pub timeline_view: WindowSize,
    pub tag_edit: WindowSize,
    pub tag_view: WindowSize,
    pub app_colours: WindowSize,
}

pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

pub const DEFAULT_WINDOW_SIZES: WindowSizes = WindowSizes {
    main_window: WindowSize {
        width: 1000.0,
        height: 700.0,
    },
    entity_edit: WindowSize {
        width: 450.0,
        height: 450.0,
    },
    entity_view: WindowSize {
        width: 250.0,
        height: 300.0,
    },
    timeline_edit: WindowSize {
        width: 400.0,
        height: 550.0,
    },
    timeline_view: WindowSize {
        width: 850.0,
        height: 700.0,
    },
    tag_edit: WindowSize {
        width: 300.0,
        height: 300.0,
    },
    tag_view: WindowSize {
        width: 300.0,
        height: 500.0,
    },
    app_colours: WindowSize {
        width: 400.0,
        height: 600.0,
    },
};

pub const DEFAULT_NEW_WINDOW_X_OFFSET_FROM_MAIN_WINDOW: f32 = 40.0;
pub const DEFAULT_NEW_WINDOW_Y_OFFSET_FROM_MAIN_WINDOW: f32 = 30.0;

pub const DESIRED_INPUT_TEXT_NUMBER_DAY_WIDTH: f32 = 30.0;
pub const DESIRED_INPUT_TEXT_NUMBER_YEAR_WIDTH: f32 = 50.0;

pub static VIEW_BUTTON_WIDTH: f32 = 30.0;
pub static EDIT_BUTTON_WIDTH: f32 = 30.0;
pub static REMOVE_BUTTON_WIDTH: f32 = 25.0;

pub static EDIT_SYMBOL: &str = "✏";
pub static VIEW_SYMBOL: &str = "👁";
