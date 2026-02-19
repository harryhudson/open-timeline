// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! The GUI for altering application colours
//!

use crate::Config;
use crate::app_colours::AppColours;
use crate::consts::DEFAULT_WINDOW_SIZES;
use crate::shortcuts::global_shortcuts;
use crate::{app::ActionRequest, app_colours::ColourTheme};
use eframe::egui::{CentralPanel, Context, Response, ScrollArea, Ui, Vec2, ViewportId};
use open_timeline_gui_core::{BreakOutWindow, CheckForUpdates, Reload, Shortcut, window_has_focus};
use open_timeline_renderer::{Colour, TimelineColours};
use tokio::sync::mpsc::UnboundedSender;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColoursChanged {
    Changed,
    Unchanged,
}

///
#[derive(Debug)]
pub struct TimelineColoursRaw {
    background_a: [u8; 3],
    background_b: [u8; 3],

    dividing_line_colour: [u8; 3],
    // dividing_line_thickness: [u8; 3],

    //
    entity_text_box_fill_colour: [u8; 3],
    // entity_text_box_border_colour: [u8; 3],
    // entity_text_box_border_thickness: [u8; 3],

    //
    entity_date_box_fill_colour: [u8; 3],
    // entity_date_box_border_colour: [u8; 3],
    // entity_date_box_border_thickness: [u8; 3],

    //
    entity_text_colour: [u8; 3],

    heading_text_colour: [u8; 3],
    heading_fill_colour: [u8; 3],
}

impl From<AppColours> for TimelineColoursRaw {
    fn from(value: AppColours) -> Self {
        Self {
            background_a: value.timeline_colours.background.a.into(),
            background_b: value.timeline_colours.background.b.into(),

            dividing_line_colour: value.timeline_colours.dividing_line.colour.into(),
            // dividing_line_thickness: value.timeline_colours.placeholder.into(),

            //
            entity_text_box_fill_colour: value.timeline_colours.entity.text_box.fill_colour.into(),
            // entity_text_box_border_colour: value.timeline_colours.placeholder.into(),
            // entity_text_box_border_thickness: value.timeline_colours.placeholder.into(),

            //
            entity_date_box_fill_colour: value.timeline_colours.entity.date_box.fill_colour.into(),
            // entity_date_box_border_colour: value.timeline_colours.placeholder.into(),
            // entity_date_box_border_thickness: value.timeline_colours.placeholder.into(),

            //
            entity_text_colour: value.timeline_colours.entity.text_colour.into(),

            heading_text_colour: value.timeline_colours.heading.text_colour.into(),
            heading_fill_colour: value.timeline_colours.heading.rect.fill_colour.into(),
        }
    }
}

impl Default for TimelineColoursRaw {
    fn default() -> Self {
        let timeline_colours = TimelineColours::default();
        Self {
            background_a: timeline_colours.background.a.into(),
            background_b: timeline_colours.background.b.into(),

            dividing_line_colour: timeline_colours.dividing_line.colour.into(),
            // dividing_line_thickness: todo!(),

            //
            entity_text_box_fill_colour: timeline_colours.entity.text_box.fill_colour.into(),
            // entity_text_box_border_colour: todo!(),
            // entity_text_box_border_thickness: todo!(),

            //
            entity_date_box_fill_colour: timeline_colours.entity.date_box.fill_colour.into(),
            // entity_date_box_border_colour: todo!(),
            // entity_date_box_border_thickness: todo!(),

            //
            entity_text_colour: timeline_colours.entity.text_colour.into(),

            heading_text_colour: timeline_colours.heading.text_colour.into(),
            heading_fill_colour: timeline_colours.heading.rect.fill_colour.into(),
        }
    }
}

///
#[derive(Debug)]
pub struct AppColoursRaw {
    text: [u8; 3],
    panel: [u8; 3],
    button_fill: [u8; 3],
    separator_lines: [u8; 3],
    text_inputs: [u8; 3],
    checkbox_and_radio: [u8; 3],
    side_panel_menu_button_fill: [u8; 3],
    side_panel_menu_button_text: [u8; 3],
    donate_button_fill: [u8; 3],
    donate_button_text: [u8; 3],
    hyperlink: [u8; 3],
}

impl From<AppColours> for AppColoursRaw {
    fn from(value: AppColours) -> Self {
        Self {
            text: value.text.into(),
            panel: value.panel.into(),
            button_fill: value.button_fill.into(),
            separator_lines: value.separator_lines.into(),
            text_inputs: value.text_inputs.into(),
            checkbox_and_radio: value.checkbox_and_radio.into(),
            side_panel_menu_button_fill: value.side_panel_menu_button_fill.into(),
            side_panel_menu_button_text: value.side_panel_menu_button_text.into(),
            donate_button_fill: value.donate_button_fill.into(),
            donate_button_text: value.donate_button_text.into(),
            hyperlink: value.hyperlink.into(),
        }
    }
}

/// Manage app colours
#[derive(Debug)]
pub struct AppColoursGui {
    ///
    colours: AppColours,

    /// Send an action request to the main loop
    tx_action_request: UnboundedSender<ActionRequest>,

    /// Send an action request to the main loop
    tx_app_colours: UnboundedSender<AppColours>,

    /// Whether this window should be closed or not
    wants_to_be_closed: bool,

    /// App colour inputs
    raw_app_colours: AppColoursRaw,

    /// Timeline colour inputs
    raw_timeline_colours: TimelineColoursRaw,
}

impl AppColoursGui {
    /// Create new AppColoursGui
    pub fn new(
        config: Config,
        tx_action_request: UnboundedSender<ActionRequest>,
        tx_app_colours: UnboundedSender<AppColours>,
    ) -> Self {
        let app_colours = match config.colour_theme {
            ColourTheme::Custom(app_colours) => app_colours,
            _ => Config::load().unwrap().custom_theme,
        };
        Self {
            colours: app_colours.clone(),
            tx_action_request,
            tx_app_colours,
            wants_to_be_closed: false,
            raw_app_colours: app_colours.into(),
            raw_timeline_colours: app_colours.into(),
        }
    }

    ///
    fn draw_app_colours_inputs(&mut self, ui: &mut Ui) -> ColoursChanged {
        // Setup colour control lables, input, and outputs
        let colour_controls = [
            (
                "Text",
                &mut self.raw_app_colours.text,
                &mut self.colours.text,
            ),
            (
                "Panels",
                &mut self.raw_app_colours.panel,
                &mut self.colours.panel,
            ),
            (
                "Buttons",
                &mut self.raw_app_colours.button_fill,
                &mut self.colours.button_fill,
            ),
            (
                "Separator lines",
                &mut self.raw_app_colours.separator_lines,
                &mut self.colours.separator_lines,
            ),
            (
                "Text inputs",
                &mut self.raw_app_colours.text_inputs,
                &mut self.colours.text_inputs,
            ),
            (
                "Checkboxes & radio buttons",
                &mut self.raw_app_colours.checkbox_and_radio,
                &mut self.colours.checkbox_and_radio,
            ),
            (
                "Side panel menu button fill",
                &mut self.raw_app_colours.side_panel_menu_button_fill,
                &mut self.colours.side_panel_menu_button_fill,
            ),
            (
                "Side panel menu button text",
                &mut self.raw_app_colours.side_panel_menu_button_text,
                &mut self.colours.side_panel_menu_button_text,
            ),
            (
                "Donate button fill",
                &mut self.raw_app_colours.donate_button_fill,
                &mut self.colours.donate_button_fill,
            ),
            (
                "Donate button text",
                &mut self.raw_app_colours.donate_button_text,
                &mut self.colours.donate_button_text,
            ),
            (
                "Hyperlinks",
                &mut self.raw_app_colours.hyperlink,
                &mut self.colours.hyperlink,
            ),
        ];

        // Draw the colour controls
        let changed_colours = colour_controls
            .into_iter()
            .map(|(label, input, target)| draw_colour_input(ui, label, input, target))
            .any(|response| response.changed());
        if changed_colours {
            ColoursChanged::Changed
        } else {
            ColoursChanged::Unchanged
        }
    }

    ///
    fn draw_timeline_colour_inputs(&mut self, ui: &mut Ui) -> ColoursChanged {
        // Setup colour control lables, input, and outputs
        let colour_controls = [
            (
                "Background A",
                &mut self.raw_timeline_colours.background_a,
                &mut self.colours.timeline_colours.background.a,
            ),
            (
                "Background B",
                &mut self.raw_timeline_colours.background_b,
                &mut self.colours.timeline_colours.background.b,
            ),
            (
                "Dividing Line",
                &mut self.raw_timeline_colours.dividing_line_colour,
                &mut self.colours.timeline_colours.dividing_line.colour,
            ),
            (
                "Entity Text Box",
                &mut self.raw_timeline_colours.entity_text_box_fill_colour,
                &mut self.colours.timeline_colours.entity.text_box.fill_colour,
            ),
            (
                "Entity Date Box",
                &mut self.raw_timeline_colours.entity_date_box_fill_colour,
                &mut self.colours.timeline_colours.entity.date_box.fill_colour,
            ),
            (
                "Entity Text",
                &mut self.raw_timeline_colours.entity_text_colour,
                &mut self.colours.timeline_colours.entity.text_colour,
            ),
            (
                "Heading Text",
                &mut self.raw_timeline_colours.heading_text_colour,
                &mut self.colours.timeline_colours.heading.text_colour,
            ),
            (
                "Heading Fill",
                &mut self.raw_timeline_colours.heading_fill_colour,
                &mut self.colours.timeline_colours.heading.rect.fill_colour,
            ),
        ];

        // Draw the colour controls
        let changed_colours = colour_controls
            .into_iter()
            .map(|(label, input, target)| draw_colour_input(ui, label, input, target))
            .any(|response| response.changed());
        if changed_colours {
            ColoursChanged::Changed
        } else {
            ColoursChanged::Unchanged
        }
    }
}

impl Reload for AppColoursGui {
    fn request_reload(&mut self) {
        // N/A
    }

    fn check_reload_response(&mut self) {
        // N/A
    }
}

impl CheckForUpdates for AppColoursGui {
    fn check_for_updates(&mut self) {
        // N/A
    }

    fn waiting_for_updates(&mut self) -> bool {
        false
    }
}

impl BreakOutWindow for AppColoursGui {
    fn draw(&mut self, ctx: &Context) {
        // Handle shortcuts
        if window_has_focus(ctx) && Shortcut::close_window(ctx) {
            self.wants_to_be_closed = true;
        }

        // Check for global shortcuts
        global_shortcuts(ctx, &mut self.tx_action_request);

        CentralPanel::default().show(ctx, |ui| {
            // Title
            open_timeline_gui_core::Label::heading(ui, "App Colours");
            ui.separator();

            //
            ScrollArea::vertical().show(ui, |ui| {
                ui.set_width(ui.available_width());

                //
                open_timeline_gui_core::Label::sub_heading(ui, "Application colours");
                let app_colours_changed = self.draw_app_colours_inputs(ui);
                ui.add_space(10.0);

                //
                open_timeline_gui_core::Label::sub_heading(ui, "Application colours");
                let timeline_colours_changed = self.draw_timeline_colour_inputs(ui);

                // Update the application colours if applicable
                if app_colours_changed == ColoursChanged::Changed
                    || timeline_colours_changed == ColoursChanged::Changed
                {
                    info!("Colours to be changed");
                    let app_colours = self.colours.clone();
                    match self.tx_app_colours.send(app_colours) {
                        Ok(()) => (),
                        Err(e) => warn!("Error sending app colours {e}"),
                    }
                }
            });
        });
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(
            DEFAULT_WINDOW_SIZES.app_colours.width,
            DEFAULT_WINDOW_SIZES.app_colours.height,
        )
    }

    fn viewport_id(&mut self) -> ViewportId {
        ViewportId(eframe::egui::Id::from("app_colours"))
    }

    fn title(&mut self) -> String {
        format!("App Colours")
    }

    fn wants_to_be_closed(&mut self) -> bool {
        self.wants_to_be_closed
    }
}

/// Draw a colour control
fn draw_colour_input(
    ui: &mut Ui,
    label: &str,
    input: &mut [u8; 3],
    target: &mut Colour,
) -> Response {
    ui.horizontal(|ui| {
        let response = ui.color_edit_button_srgb(input);
        ui.label(label);
        *target = (*input).into();
        response
    })
    .inner
}
