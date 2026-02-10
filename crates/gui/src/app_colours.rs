// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Themes and colours for the OpenTimeline desktop app
//!

use eframe::egui::{Color32, Context, Stroke, Theme, Visuals};
use open_timeline_renderer::{
    BackgroundColours, BoxStyle, Colour, EntityStyle, HeadingStyle, LineStyle, TimelineColours,
    TimelineEntityColourModifier,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColourTheme {
    System,
    Light,
    Dark,
    Siphonophore,
    Custom(AppColours),
}

impl ColourTheme {
    pub fn timeline_colours(&self, ctx: &Context) -> TimelineColours {
        // Helper: light mode timeline colours
        fn light() -> TimelineColours {
            TimelineColours::default()
        }

        // Helper: dark mode timeline colours
        fn dark() -> TimelineColours {
            let mut colours = TimelineColours::default();
            colours.background.a = Colour::from_rgb(50, 50, 50);
            colours.background.b = Colour::from_rgb(30, 30, 30);
            colours
        }

        // Get the correct timeline colour
        match *self {
            ColourTheme::Light => light(),
            ColourTheme::Dark => dark(),
            ColourTheme::System => {
                if ctx.style().visuals.dark_mode {
                    dark()
                } else {
                    light()
                }
            }
            ColourTheme::Siphonophore => AppColours::siphonophore_theme().timeline_colours,
            ColourTheme::Custom(app_colours) => app_colours.timeline_colours,
        }
    }
}

/// Colours for the desktop GUI
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppColours {
    /// The colour of text
    pub text: Colour,

    /// The colour of panels
    pub panel: Colour,

    /// The colour of radio buttons and the like
    pub button_fill: Colour,

    ///
    pub separator_lines: Colour,

    ///
    pub text_inputs: Colour,

    ///
    pub checkbox_and_radio: Colour,

    ///
    pub side_panel_menu_button_fill: Colour,

    ///
    pub side_panel_menu_button_text: Colour,

    ///
    pub donate_button_fill: Colour,

    ///
    pub donate_button_text: Colour,

    ///
    pub hyperlink: Colour,

    ///
    pub timeline_colours: TimelineColours,
}

impl AppColours {
    pub fn use_theme(ctx: &Context, theme: ColourTheme) {
        match theme {
            ColourTheme::System => Self::use_system_theme(ctx),
            ColourTheme::Light => Self::use_default_light_theme(ctx),
            ColourTheme::Dark => Self::use_default_dark_theme(ctx),
            ColourTheme::Siphonophore => Self::use_siphonophore_theme(ctx),
            ColourTheme::Custom(app_colours) => Self::use_custom_theme(ctx, app_colours),
        }
    }

    fn siphonophore_theme() -> Self {
        Self {
            text: Colour::from_rgb(153, 202, 228),
            panel: Colour::from_rgb(21, 51, 65),
            button_fill: Colour::from_rgb(48, 84, 99),
            separator_lines: Colour::from_rgb(56, 84, 102),
            text_inputs: Colour::from_rgb(46, 79, 91),
            checkbox_and_radio: Colour::from_rgb(52, 81, 100),
            side_panel_menu_button_fill: Colour::from_rgb(43, 77, 90),
            side_panel_menu_button_text: Colour::from_rgb(143, 198, 214),
            donate_button_fill: Colour::from_rgb(87, 163, 118),
            donate_button_text: Colour::from_rgb(31, 44, 48),
            hyperlink: Colour::from_rgb(102, 204, 157),
            timeline_colours: TimelineColours {
                background: BackgroundColours {
                    a: Colour::from_rgb(52, 100, 110),
                    b: Colour::from_rgb(44, 78, 89),
                },
                dividing_line: LineStyle {
                    colour: Colour::from_rgb(71, 125, 127),
                    thickness: 0.5,
                },
                entity: EntityStyle {
                    text_box: BoxStyle {
                        fill_colour: { Colour::from_rgb(73, 118, 133) },
                        border: None,
                    },
                    date_box: BoxStyle {
                        fill_colour: { Colour::from_rgb(85, 158, 164) },
                        border: None,
                    },
                    text_colour: Colour::from_rgb(0, 0, 0),
                    click_colour: TimelineEntityColourModifier::Lighten,
                    hover_colour: TimelineEntityColourModifier::Lighten,
                },
                heading: HeadingStyle {
                    rect: BoxStyle {
                        fill_colour: Colour::from_rgb(38, 38, 65),
                        border: None,
                    },
                    text_colour: Colour::from_rgb(134, 189, 213),
                },
            },
        }
    }

    /// Use the system theme (dark or light).  Defaults to light
    fn use_system_theme(ctx: &Context) {
        match ctx.system_theme() {
            Some(Theme::Light) => Self::use_default_light_theme(ctx),
            Some(Theme::Dark) => Self::use_default_dark_theme(ctx),
            None => Self::use_default_light_theme(ctx),
        }
    }

    fn use_default_light_theme(ctx: &Context) {
        ctx.style_mut(|style| style.visuals = Visuals::light());
    }

    fn use_default_dark_theme(ctx: &Context) {
        ctx.style_mut(|style| style.visuals = Visuals::dark());
    }

    fn use_siphonophore_theme(ctx: &Context) {
        Self::siphonophore_theme().set(ctx);
    }

    fn use_custom_theme(ctx: &Context, app_colours: AppColours) {
        app_colours.set(ctx);
    }

    fn set(&self, ctx: &Context) {
        Self::use_default_light_theme(ctx);
        ctx.style_mut(|style| {
            // Background
            style.visuals.window_fill = self.panel.into();
            style.visuals.panel_fill = self.panel.into();

            // Text colour
            style.visuals.override_text_color = None;
            style.visuals.widgets.active.fg_stroke.color = self.text.into();
            style.visuals.widgets.hovered.fg_stroke.color =
                Colour::lightened_colour(self.text).into();
            style.visuals.widgets.inactive.fg_stroke.color = self.text.into();
            style.visuals.widgets.noninteractive.fg_stroke.color = self.text.into();
            style.visuals.widgets.open.fg_stroke.color = self.text.into();

            // Buttons
            style.visuals.widgets.active.weak_bg_fill = self.button_fill.into();
            style.visuals.widgets.hovered.weak_bg_fill =
                Colour::lightened_colour(self.button_fill).into();
            style.visuals.widgets.inactive.weak_bg_fill = self.button_fill.into();
            style.visuals.widgets.noninteractive.weak_bg_fill = self.button_fill.into();
            style.visuals.widgets.open.weak_bg_fill = self.button_fill.into();

            // Checkboxes & radio buttons
            style.visuals.widgets.active.bg_fill = self.checkbox_and_radio.into();
            style.visuals.widgets.hovered.bg_fill =
                Colour::lightened_colour(self.checkbox_and_radio).into();
            style.visuals.widgets.inactive.bg_fill = self.checkbox_and_radio.into();
            style.visuals.widgets.noninteractive.bg_fill = self.checkbox_and_radio.into();
            style.visuals.widgets.open.bg_fill = self.checkbox_and_radio.into();

            // Side panel menu buttons
            style.visuals.selection.bg_fill = self.side_panel_menu_button_fill.into();
            style.visuals.selection.stroke.color = self.side_panel_menu_button_text.into();

            // Hyperlinks
            style.visuals.hyperlink_color = self.hyperlink.into();

            // Separator lines
            style.visuals.widgets.noninteractive.bg_stroke = Stroke {
                width: 1.0,
                color: self.separator_lines.into(),
            };

            // Inputs
            style.visuals.text_edit_bg_color = Some(self.text_inputs.into());
        });
    }

    pub const fn default_donate_button_fill() -> Color32 {
        Color32::LIGHT_GREEN
    }

    pub const fn default_donate_button_text_colour() -> Color32 {
        Color32::BLACK
    }

    pub const fn default_hyperlink_text_colour() -> Color32 {
        Color32::DARK_BLUE
    }
}

impl Default for AppColours {
    fn default() -> Self {
        Self {
            text: Colour::from_hex("#005064").unwrap(),
            panel: Colour::from_hex("#fefcf6ff").unwrap(),
            button_fill: Colour::from_hex("#00CCCC").unwrap(),
            separator_lines: Colour::from_rgb(100, 100, 100),
            text_inputs: Colour::from_rgb(225, 225, 225),
            checkbox_and_radio: Colour::from_rgb(175, 175, 175),
            side_panel_menu_button_fill: Colour::from_rgb(175, 175, 175),
            side_panel_menu_button_text: Colour::from_rgb(25, 25, 25),
            donate_button_fill: Self::default_donate_button_fill().into(),
            donate_button_text: Self::default_donate_button_text_colour().into(),
            hyperlink: Self::default_hyperlink_text_colour().into(),
            timeline_colours: TimelineColours::default(),
        }
    }
}
