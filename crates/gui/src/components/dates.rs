// SPDX-License-Identifier: GPL-3.0-or-later

//!
//! Everything for GUI dates
//!

use crate::common::ToOpenTimelineType;
use crate::consts::*;
use eframe::egui::{Align, Color32, ComboBox, Context, Layout, TextEdit, Ui, Vec2};
use open_timeline_core::Date;
use open_timeline_crud::CrudError;
use open_timeline_gui_core::{
    Draw, Valid, ValidAsynchronous, ValidSynchronous, ValidityAsynchronous, ValiditySynchronous,
    ValitityStatus, conform_string_input_to_int_in_range,
};

/// Used to represent/indicate a start or end date
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum StartEnd {
    Start,
    End,
}

#[derive(Debug)]
pub struct DatesGui {
    /// The start year input
    start_year: String,

    /// The start month input
    start_month: usize,

    /// The start dat input
    start_day: String,

    /// The end year input
    end_year: String,

    /// The end month input
    end_month: usize,

    /// The end day input
    end_day: String,

    /// The validity of the dates as considered together (not just individually
    /// - e.g. is the end equal to or after the start as it should be)
    validity: ValitityStatus<(), CrudError>,
}

impl DatesGui {
    /// Create new DatesGui
    pub fn new() -> Self {
        let mut new = Self {
            // Start
            start_year: String::new(),
            start_month: 0,
            start_day: String::new(),

            // End
            end_year: String::new(),
            end_month: 0,
            end_day: String::new(),

            // Validity
            validity: ValitityStatus::from(ValiditySynchronous::Valid, Some(Ok(()))),
        };
        new.update_validity();
        new
    }

    fn draw_date(&mut self, _ctx: &Context, ui: &mut Ui, start_or_end: StartEnd) {
        let dates_validity = self.validity();
        let mut update_validity = false;

        let months = [
            "",
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];

        // Start or end label
        let label = match start_or_end {
            StartEnd::Start => String::from("Start"),
            StartEnd::End => String::from("End"),
        };

        // Get mut references to the data input buffers
        let (day_buf, month_buf, year_buf) = match start_or_end {
            StartEnd::Start => (
                &mut self.start_day,
                &mut self.start_month,
                &mut self.start_year,
            ),
            StartEnd::End => (&mut self.end_day, &mut self.end_month, &mut self.end_year),
        };

        ui.push_id(&start_or_end, |ui| {
            ui.vertical(|ui| {
                // Draw the start or end subheading
                open_timeline_gui_core::Label::sub_heading(ui, &label);

                // Indicate what each input box is for
                ui.label("day / month / year");

                ui.horizontal(|ui| {
                    // Set styling
                    if let ValidityAsynchronous::Invalid(_) = dates_validity {
                        ui.visuals_mut().override_text_color = Some(Color32::WHITE);

                        ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::LIGHT_RED;
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::LIGHT_RED;
                        ui.style_mut().visuals.widgets.open.weak_bg_fill = Color32::LIGHT_RED;
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::LIGHT_RED;
                    } else {
                        // TODO: this is the colour of the button that appears when not interacting
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
                    }

                    // Day
                    let day_input = if self.validity.synchronous() == ValiditySynchronous::Valid {
                        TextEdit::singleline(day_buf)
                            .desired_width(DESIRED_INPUT_TEXT_NUMBER_DAY_WIDTH)
                    } else {
                        TextEdit::singleline(day_buf)
                            .desired_width(DESIRED_INPUT_TEXT_NUMBER_DAY_WIDTH)
                            .background_color(Color32::LIGHT_RED)
                            .text_color(Color32::WHITE)
                    };
                    if ui.add(day_input).changed() {
                        conform_string_input_to_int_in_range(day_buf, 1..=31);
                        update_validity = true;
                    };

                    // Month
                    let month_changed = ComboBox::from_id_salt(format!("{start_or_end:?}_month"))
                        .selected_text(months[*month_buf])
                        .show_ui(ui, |ui| {
                            let mut changed = false;
                            for (i, month) in months.iter().enumerate() {
                                if ui.selectable_value(month_buf, i, *month).changed() {
                                    changed = true;
                                }
                            }
                            changed
                        })
                        .inner;
                    if let Some(true) = month_changed {
                        update_validity = true;
                    }

                    // Year
                    let year_input = if self.validity.synchronous() == ValiditySynchronous::Valid {
                        TextEdit::singleline(year_buf)
                            .desired_width(DESIRED_INPUT_TEXT_NUMBER_YEAR_WIDTH)
                    } else {
                        TextEdit::singleline(year_buf)
                            .desired_width(DESIRED_INPUT_TEXT_NUMBER_YEAR_WIDTH)
                            .background_color(Color32::LIGHT_RED)
                            .text_color(Color32::WHITE)
                    };
                    if ui.add(year_input).changed() {
                        conform_string_input_to_int_in_range(
                            year_buf,
                            (open_timeline_core::MIN_YEAR as isize)
                                ..=(open_timeline_core::MAX_YEAR as isize),
                        );
                        debug!("Year changed");
                        update_validity = true;
                    }
                });
            });
        });

        if update_validity {
            debug!("Updating date validity");
            self.update_validity();
        }
    }
}

impl ValidSynchronous for DatesGui {
    fn is_valid_synchronous(&self) -> bool {
        self.validity.synchronous() == ValiditySynchronous::Valid
    }

    fn update_validity_synchronous(&mut self) {
        debug!("Updating date sync validity");

        // Start
        let start_date = match validate_start(
            self.start_day.clone(),
            self.start_month,
            self.start_year.clone(),
        ) {
            StartDateValidity::Valid(date) => {
                debug!("Start date invalid");
                date
            }
            StartDateValidity::Invalid(error_msg) => {
                debug!("Start date invalid: {error_msg}");
                self.validity
                    .set_synchronous(ValiditySynchronous::Invalid(error_msg));
                return;
            }
        };

        // End
        let end_date =
            match validate_end(self.end_day.clone(), self.end_month, self.end_year.clone()) {
                EndDateValidity::ValidNoDate => {
                    debug!("End date is valid (no date)");
                    None
                }
                EndDateValidity::ValidDate(date) => {
                    debug!("End date is valid");
                    Some(date)
                }
                EndDateValidity::Invalid(error_msg) => {
                    debug!("End date is invalid");
                    self.validity
                        .set_synchronous(ValiditySynchronous::Invalid(error_msg));
                    return;
                }
            };

        // Both
        if let Some(end_date) = end_date {
            if start_date > end_date {
                self.validity
                    .set_synchronous(ValiditySynchronous::Invalid(String::from(
                        "The start date must be before the end date",
                    )));
                return;
            }
        }

        // Otherwise it's valid
        debug!("Dates are valid");
        self.validity.set_synchronous(ValiditySynchronous::Valid);
    }

    fn validity_synchronous(&self) -> ValiditySynchronous {
        self.validity.synchronous()
    }
}

impl ValidAsynchronous for DatesGui {
    type Error = CrudError;

    fn check_for_asynchronous_validity_response(&mut self) {
        //
    }

    fn is_valid_asynchronous(&self) -> Option<Result<(), Self::Error>> {
        Some(Ok(()))
    }

    fn trigger_asynchronous_validity_update(&mut self) {
        //
    }
}

impl Valid for DatesGui {}

impl ToOpenTimelineType<(Date, Option<Date>)> for DatesGui {
    // TODO: can we reuse some of the validation checking stuff
    fn to_opentimeline_type(&self) -> (Date, Option<Date>) {
        // Start
        let start_day: Option<i64> = self.start_day.trim().parse::<i64>().ok();
        let start_month: Option<i64> = (1..=12)
            .contains(&self.start_month)
            .then_some(self.start_month as i64);
        let start_year: i64 = self.start_year.trim().parse::<i64>().unwrap();
        let start = Date::from(start_day, start_month, start_year).unwrap();

        // End
        let end_day: Option<i64> = self.end_day.trim().parse::<i64>().ok();
        let end_month: Option<i64> = (1..=12)
            .contains(&self.end_month)
            .then_some(self.end_month as i64);

        match self.end_year.trim().parse::<i64>() {
            Err(_) => (start, None),
            Ok(end_year) => {
                let end = Date::from(end_day, end_month, end_year).unwrap();
                (start, Some(end))
            }
        }
    }
}

impl Draw for DatesGui {
    fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        let size = Vec2::new(ui.available_width(), 0.0);
        ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
            self.draw_date(ctx, ui, StartEnd::Start);
            ui.separator();
            self.draw_date(ctx, ui, StartEnd::End);
        });
    }
}

impl From<(Date, Option<Date>)> for DatesGui {
    fn from((start, end): (Date, Option<Date>)) -> Self {
        // Start
        let (start_day, start_month, start_year) = gui_date_components_from_date(start);

        // End
        let (end_day, end_month, end_year) = match end {
            Some(date) => gui_date_components_from_date(date),
            None => (String::new(), 0, String::new()),
        };

        Self {
            start_year,
            start_month,
            start_day,
            end_year,
            end_month,
            end_day,
            validity: ValitityStatus::from(ValiditySynchronous::Valid, Some(Ok(()))),
        }
    }
}

/// Convert a date to its constituents and into types needed to display it in
/// the GUI
fn gui_date_components_from_date(date: Date) -> (String, usize, String) {
    let year = date.year().to_string();
    let month = match date.month() {
        Some(month) => month.value(),
        None => 0,
    };
    let day = match date.day() {
        Some(day) => day.to_string(),
        None => String::new(),
    };
    (day, month.into(), year)
}

/// Representation of the possible validity states of a start date
enum StartDateValidity {
    Valid(Date),
    Invalid(String),
}

/// Representation of the possible validity states of an end date
enum EndDateValidity {
    ValidDate(Date),
    ValidNoDate,
    Invalid(String),
}

/// Validate a start date (must have a year component)
fn validate_start(day: String, month: usize, year: String) -> StartDateValidity {
    // Parse
    let start_day: Option<i64> = day.trim().parse::<i64>().ok();
    let start_month: Option<i64> = (1..=12).contains(&month).then_some(month as i64);
    let Ok(start_year) = year.trim().parse::<i64>() else {
        return StartDateValidity::Invalid(String::from("No start year"));
    };

    // Validate
    match Date::from(start_day, start_month, start_year) {
        Ok(date) => StartDateValidity::Valid(date),
        Err(e) => StartDateValidity::Invalid(e.to_string()),
    }
}

/// Validate an end date (needn't have a year component)
fn validate_end(day: String, month: usize, year: String) -> EndDateValidity {
    // Parse
    let end_day: Option<i64> = day.trim().parse::<i64>().ok();
    let end_month: Option<i64> = (1..=12).contains(&month).then_some(month as i64);
    let end_year: Option<i64> = year.trim().parse::<i64>().ok();

    // Validate
    match (end_day, end_month, end_year) {
        (None, None, None) => return EndDateValidity::ValidNoDate,
        (_, _, None) => return EndDateValidity::Invalid(String::from("End year is empty")),
        _ => (),
    }
    match Date::from(end_day, end_month, end_year.unwrap()) {
        Ok(date) => EndDateValidity::ValidDate(date),
        Err(e) => EndDateValidity::Invalid(e.to_string()),
    }
}
