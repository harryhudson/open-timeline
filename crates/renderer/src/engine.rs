// SPDX-License-Identifier: MIT

//!
//! The `open-timeline-renderer` engine
//!

mod colours;
mod consts;
mod date_range;
mod entity;
mod events;
mod heading;
mod helpers;
mod layout_params;
mod point;
mod primitives;

pub(crate) use date_range::*;
pub(crate) use helpers::*;
pub(crate) use layout_params::*;

pub use colours::*;
pub use consts::*;
pub use entity::*;
pub use events::*;
pub use heading::*;
use log::{debug, trace};
pub use point::*;
pub use primitives::*;

use crate::colour::Colour;
use bool_tag_expr::BoolTagExpr;
use open_timeline_core::{Date, Day, Entity, HasIdAndName, Month, OpenTimelineId, Year};
use std::collections::BTreeSet;

/// The core `open-timeline-renderer` engine.  This manages all entities,
/// calculations, measurements, interactions, etc, common to all timeline
/// engines (e.g. the desktop and HTML canvas engines)
pub struct Engine {
    /// The information required for all entity-related calculations
    working_entities: Vec<WorkingEntity>,

    /// The boolean tag expression to filter entities by (if any)
    entity_filter: Option<BoolTagExpr>,

    /// The timeline headings (e.g. decades)
    headings: Vec<Heading>,

    /// The function supplied to the timeline that it can use to measure text.
    ///
    /// The timeline passes the function the pixel font size and the string, and
    /// the function returns the height and width of the text.
    ///
    /// i.e. `function(font_size, text) -> (height, width)`
    measure_text_fn: Box<dyn Fn(f64, String) -> (f64, f64)>,

    /// The timelines date range (e.g. min/max year/decade and the number of
    /// decades)
    date_range: TimelineDateRange,

    /// The IDs of the entities currently selected
    ids_of_selected_entities: Vec<OpenTimelineId>, // TODO: add public API

    /// The timeline's colours
    colours: TimelineColours,

    /// The timeline's global offset.  This is never scaled.
    offset: TimelineOffset,

    /// The timeline's zoom level
    zoom: f64,

    // TODO: type this with helpers for setting limits
    /// The timeline's datetime scale factor (stretch in x-direction)
    datetime_scale: f64,

    /// These layout parameters are measured using the `measure_text_fn`
    measured_layout_params: MeasuredLayoutParams,

    /// These fixed params can be set directly, but they're not altered when,
    /// for example, zooming.  These are to be set by users directly.
    fixed_layout_params: ScalableLayoutParams,

    /// These are the fixed layout params altered by zooming.  These are not to
    /// be set by users directly - they are derived/calculated from the fixed
    /// values.
    zoomed_layout_params: ScalableLayoutParams,

    /// All interaction events that an external programme might be interested in
    interaction_events: Vec<TimelineInteractionEvent>,

    /// Whether the text of an entity should stick to the left of the screen
    /// rather than disappear off it (space allowing)
    sticky_text: bool,

    /// The size of the canvas
    canvas_size: Point,
}

impl Engine {
    /// Create a new engine.  Pass in a function that the engine can call to
    /// measure text
    pub fn new<T>(measure_text_fn: T) -> Self
    where
        T: 'static + Fn(f64, String) -> (f64, f64),
    {
        Self {
            working_entities: Vec::new(),
            entity_filter: None,
            headings: Vec::new(),
            measure_text_fn: Box::new(measure_text_fn),
            date_range: TimelineDateRange::default(),
            ids_of_selected_entities: Vec::new(),
            colours: TimelineColours::default(),
            offset: TimelineOffset::default(),
            zoom: 1.0,
            datetime_scale: MIN_DATETIME_SCALE,
            measured_layout_params: MeasuredLayoutParams::default(),
            fixed_layout_params: ScalableLayoutParams::default(),
            zoomed_layout_params: ScalableLayoutParams::default(),
            interaction_events: Vec::new(),
            sticky_text: true,
            canvas_size: Point { x: 0.0, y: 0.0 },
        }
    }

    pub fn ids_of_selected_entities(&self) -> &Vec<OpenTimelineId> {
        &self.ids_of_selected_entities
    }

    pub fn set_ids_of_selected_entities(&mut self, entity_ids: Vec<OpenTimelineId>) {
        self.ids_of_selected_entities = entity_ids
    }

    pub fn add_id_of_selected_entity(&mut self, entity_id: OpenTimelineId) {
        self.ids_of_selected_entities.push(entity_id);
    }

    pub fn clear_ids_of_selected_entities(&mut self) {
        self.ids_of_selected_entities.clear()
    }

    pub fn remove_id_from_selected_entities_list(&mut self, entity_id: OpenTimelineId) {
        self.ids_of_selected_entities.retain(|id| *id != entity_id)
    }

    /// Get the current zoom level
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// Get the current datetime scale factor
    pub fn datetime_scale(&self) -> f64 {
        self.datetime_scale
    }

    /// Get the timeline colours
    pub fn colours(&self) -> TimelineColours {
        self.colours
    }

    ///
    pub fn set_colours(&mut self, colours: TimelineColours) {
        debug!("engine set colours");
        self.colours = colours;
    }

    /// Calculate the width of the string
    fn str_width(&self, str: &str) -> f64 {
        (self.measure_text_fn)(self.zoomed_layout_params.font_size_px, str.to_string()).0
    }

    /// Calculate the height of the string
    fn str_height(&self, str: &str) -> f64 {
        (self.measure_text_fn)(self.zoomed_layout_params.font_size_px, str.to_string()).1
    }

    /// To be called when the text size changes (e.g. font size changed, or
    /// zoom changed, etc).  Calculates the height of a row and the width of a
    /// year using the `measure_text_fn`
    fn update_measured_layout_params(&mut self) {
        // Calculate the row height due to text
        let row_height = self.str_height("lpfHT");
        self.measured_layout_params.row_height_no_padding = row_height;

        // Calculate the year width due to heading width and padding
        let decade_str_width = self.str_width("1234s");

        // Apply X scaling
        let decade_str_width = decade_str_width * self.datetime_scale;

        // Set year width
        self.measured_layout_params.year_width =
            (decade_str_width + (self.zoomed_layout_params.padding_x * 2.0)) / 10.0;
    }

    /// To be called when the zoom level is changed
    fn update_zoomed_layout_params(&mut self) {
        self.zoomed_layout_params = ScalableLayoutParams {
            row_margin: self.fixed_layout_params.row_margin * self.zoom,
            min_inline_spacing: self.fixed_layout_params.min_inline_spacing * self.zoom,
            padding_x: self.fixed_layout_params.padding_x * self.zoom,
            padding_y: self.fixed_layout_params.padding_y * self.zoom,
            font_size_px: self.fixed_layout_params.font_size_px * self.zoom,
            dividing_line_thickness: self.fixed_layout_params.dividing_line_thickness * self.zoom,
            entity_highlight_thickness: self.fixed_layout_params.entity_highlight_thickness
                * self.zoom,
        };
    }

    // TODO: this must be set before drawing because otherwise the engine thinks
    // everything is out of frame - this needs to be better inforced because it
    // isn't obvious
    /// Adjust the global offset by some delta
    pub fn set_canvas_max(&mut self, x: f64, y: f64) {
        self.canvas_size = Point { x, y };
    }

    /// Adjust the global offset by some delta
    pub fn add_to_global_offset(&mut self, x_delta: f64, y_delta: f64) {
        trace!("add_to_global_offset {}, {}", x_delta, y_delta);
        self.offset.x += x_delta;
        self.offset.y += y_delta;
        self.clamp_global_offset();
    }

    /// Get all information needed to draw the timeline entities
    pub fn entities_for_drawing(&self) -> Vec<EntityOut> {
        let header_height = self.measured_layout_params.row_height_no_padding
            + (2.0 * self.zoomed_layout_params.padding_y);

        // Auto y offset as additional header shown as a consequence of x scaling
        let y_offset = if self.datetime_scale() > DATETIME_SCALE_THRESHOLD_SHOW_YEARS {
            self.offset.y + header_height
        } else {
            self.offset.y
        };

        // Combine: end, start, year_width, x_offset, y_offset, row_margin, row_height, padding
        self.working_entities
            .clone()
            .into_iter()
            .filter(|entity| !entity.is_filtered_out())
            .map(|mut entity| {
                // Text
                entity.text.colour = self.colours.entity.text_colour;

                // Text box
                entity.text_box.fill_colour = self.colours.entity.text_box.fill_colour;
                entity.text_box.border_style = self.colours.entity.text_box.border;

                // Date box
                entity.date_box.fill_colour = self.colours.entity.date_box.fill_colour;
                entity.date_box.border_style = self.colours.entity.date_box.border;

                // Return entity
                entity
            })
            .map(|mut entity| {
                if entity.is_hovered_over {
                    entity.text_box.fill_colour =
                        Colour::lightened_colour(entity.text_box.fill_colour);
                    entity.date_box.fill_colour =
                        Colour::lightened_colour(entity.date_box.fill_colour);
                    entity.text.colour = Colour::lightened_colour(entity.text.colour);
                }
                let mut entity = entity.clone_with_added_offset(self.offset.x, y_offset);
                if self.sticky_text {
                    entity.adjust_sticky_text(self.zoomed_layout_params.padding_x);
                }
                entity
            })
            .filter(|entity| {
                let date_box_min = entity.date_box.position_and_size.position;
                let date_box_max = Point {
                    x: entity.date_box.position_and_size.max_x(),
                    y: entity.date_box.position_and_size.max_y(),
                };

                let text_box_min = entity.text_box.position_and_size.position;
                let text_box_max = Point {
                    x: entity.text_box.position_and_size.max_x(),
                    y: entity.text_box.position_and_size.max_y(),
                };

                let min = date_box_min.min(text_box_min);
                let max = date_box_max.max(text_box_max);

                is_visible(min, max, self.canvas_size)
            })
            .map(|entity| entity.into())
            .collect()
    }

    // TODO: should just be &self
    /// Get all information needed to draw the timeline headings
    pub fn headings_for_drawing(&mut self) -> Vec<Heading> {
        self.update_headings();

        // Add offset to headings
        self.headings
            .clone()
            .into_iter()
            .map(|mut heading| heading.add_offset(self.offset.x))
            .filter(|heading| {
                let min = heading.text_box.position_and_size.position;
                let max = Point {
                    x: heading.text_box.position_and_size.max_x(),
                    y: heading.text_box.position_and_size.max_x(),
                };
                is_visible(min, max, self.canvas_size)
            })
            .collect()
    }

    /// Get all information needed to draw the timeline deliminating lines
    pub fn lines_for_drawing(&self) -> Vec<VerticalLine> {
        // All lines
        let mut lines = Vec::new();

        // Width of a decade
        let decade_width = self.decade_width();

        // Loop over each decade
        for decade_number in 0..=self.date_range.decade_count {
            let decade_min_x = (f64::from(decade_number) * decade_width) + self.offset.x;

            // Push the decade-dividing line
            lines.push(VerticalLine {
                x: decade_min_x,
                style: LineStyle {
                    colour: self.colours.dividing_line.colour,
                    thickness: self.zoomed_layout_params.dividing_line_thickness,
                },
            });

            // If year-dividing lines are to be shown
            if self.datetime_scale() > DATETIME_SCALE_THRESHOLD_SHOW_YEAR_LINES_PARTAL
                && decade_number != self.date_range.decade_count
            {
                // Lighten the line colour
                let mut colour = self.colours.dividing_line.colour;
                colour = Colour::lightened_colour(colour);
                colour = Colour::lightened_colour(colour);
                if self.datetime_scale() < DATETIME_X_THRESHOLD_SHOW_YEAR_LINES_FULL {
                    let factor =
                        ((DATETIME_X_THRESHOLD_SHOW_YEAR_LINES_FULL - self.datetime_scale()) / 0.5)
                            .round() as i32;
                    for _ in 0..factor {
                        colour = Colour::lightened_colour(colour);
                    }
                }

                // Width of a year
                let year_width = self.decade_width() / 10.0;

                // Loop over each year in the decade
                for year_number in 1..10 {
                    // Push the year-dividing line
                    lines.push(VerticalLine {
                        x: decade_min_x + (year_width * year_number as f64),
                        style: LineStyle {
                            colour,
                            thickness: self.zoomed_layout_params.dividing_line_thickness,
                        },
                    });
                }
            }
        }
        lines
    }

    /// Get all information needed to draw the timeline backgrounds
    pub fn backgrounds_for_drawing(&self) -> Vec<Background> {
        let mut backgrounds = Vec::new();
        for decade_number in 0..self.date_range.decade_count {
            let decade = self.date_range.decade_range_start + decade_number * 10;
            let colour_background = (decade / 100) % 2 == 0;
            let width = self.decade_width();
            let decade_number: f64 = decade_number.into();
            let x = (decade_number * width) + self.offset.x;
            let colour = if colour_background {
                self.colours.background.a
            } else {
                self.colours.background.b
            };
            backgrounds.push(Background { x, width, colour });
        }
        backgrounds
    }

    /// Get all events for dispatching & handling
    pub fn drain_interaction_events(&mut self) -> std::vec::Drain<'_, TimelineInteractionEvent> {
        self.interaction_events.drain(..)
    }

    pub fn click_on_entity(&mut self, entity_id: OpenTimelineId) {
        self.interaction_events
            .push(TimelineInteractionEvent::SingleClick(entity_id));
    }

    pub fn double_click_on_entity(&mut self, entity_id: OpenTimelineId) {
        self.interaction_events
            .push(TimelineInteractionEvent::DoubleClick(entity_id));
    }

    pub fn triple_click_on_entity(&mut self, entity_id: OpenTimelineId) {
        self.interaction_events
            .push(TimelineInteractionEvent::TripleClick(entity_id));
    }

    pub fn hover_over_entity(&mut self, entity_id: Option<OpenTimelineId>) {
        match entity_id {
            Some(entity_id) => {
                // Add to the event list
                self.interaction_events
                    .push(TimelineInteractionEvent::Hover(entity_id));

                // Update the entity
                for entity in self.working_entities.iter_mut() {
                    entity.is_hovered_over = entity.entity.id().unwrap() == entity_id;
                }
            }
            None => {
                for entity in self.working_entities.iter_mut() {
                    entity.is_hovered_over = false;
                }
            }
        }
    }

    pub fn select_entities(&mut self, entities: Vec<OpenTimelineId>) {
        let entities: BTreeSet<OpenTimelineId> = entities.into_iter().collect();
        for entity in self.working_entities.iter_mut() {
            if entities.contains(&entity.entity.id().unwrap()) {
                entity.is_selected = true;
            }
        }
    }

    // Min date only, max date only, min and max, auto
    pub fn set_date_limits(&mut self, start: Option<Date>, end: Option<Date>) {
        self.date_range.start_date_cutoff = start;
        self.date_range.end_date_cutoff = end;
        self.re_calculate();
    }

    ///
    pub fn set_sticky_text(&mut self, sticky_text: bool) {
        self.sticky_text = sticky_text;
    }

    // TODO: rename (returns decade floor & ceil years, not dates)
    /// Get the timeline's earliest and latest dates
    pub fn start_and_end_dates(&self) -> (i32, i32) {
        (
            self.date_range.decade_range_start,
            self.date_range.decade_range_end,
        )
    }

    /// Get the current date limts
    pub fn date_limits(&self) -> (Option<Date>, Option<Date>) {
        (
            self.date_range.start_date_cutoff,
            self.date_range.end_date_cutoff,
        )
    }

    pub fn set_font_size_px(&mut self, font_size_px: f64) {
        self.fixed_layout_params.font_size_px = font_size_px;
        self.update_zoomed_layout_params();
        self.re_calculate();
    }

    pub fn set_layout_params(&mut self, layout_params: ScalableLayoutParams) {
        self.fixed_layout_params = layout_params;
        self.update_zoomed_layout_params();
        self.re_calculate();
    }

    /// Accounts for zooming (might not be the same as the font size that is set
    /// using `.set_font_size_px()`)
    pub fn effective_font_size_px(&self) -> f64 {
        self.zoomed_layout_params.font_size_px
    }

    pub fn remove_entities(&mut self, to_remove: Vec<OpenTimelineId>) {
        let to_remove: BTreeSet<OpenTimelineId> = to_remove.into_iter().collect();
        self.working_entities
            .retain(|entity| !to_remove.contains(&entity.entity.id().unwrap()));
        self.re_calculate();
    }

    pub fn clear_entities(&mut self) {
        self.working_entities.clear();
        self.re_calculate();
    }

    /// The number of entities (filtering ignore)
    pub fn entity_count(&self) -> usize {
        self.working_entities.len()
    }

    // TODO: Merge in the new entities, ignoring any duplicates
    /// Add new entities to the timeline (ignores duplicates)
    pub fn add_entities(&mut self, entities: Vec<Entity>) {
        for entity in entities {
            let text_width = self.str_width(&entity.name().to_string());
            let entity_working = WorkingEntity::from(
                entity,
                self.colours,
                self.measured_layout_params,
                self.zoomed_layout_params,
                text_width,
            );
            self.working_entities.push(entity_working);
        }
        debug!("about to sort entities");
        self.sort_entities();
        self.re_calculate();
    }

    /// Overwrite the list of entities drawn on the timeline
    pub fn set_entities(&mut self, entities: Vec<Entity>) {
        self.clear_entities();
        self.add_entities(entities);
    }

    /// Set the engine to filter entities by the given tag bool expression
    pub fn set_tag_bool_expr_entity_filter(&mut self, tag_bool_expr: BoolTagExpr) {
        self.entity_filter = Some(tag_bool_expr);
        self.re_calculate();
    }

    /// Remove the entity tag bool expression filter
    pub fn remove_tag_bool_expr_entity_filter(&mut self) {
        self.entity_filter = None;
        self.re_calculate();
    }

    /// Re-run all calculations
    fn re_calculate(&mut self) {
        self.update_entities_filtered();
        self.update_timeline_date_range();
        self.update_measured_layout_params();

        let mut cloned = self.working_entities.clone();
        for entity in cloned.iter_mut() {
            let text_width = self.str_width(&entity.entity.name().to_string());
            entity.update_if_appropriate(
                self.colours,
                self.measured_layout_params,
                self.zoomed_layout_params,
                text_width,
            );
        }
        self.working_entities = cloned;
        self.calculate_entity_positions();
    }

    fn update_entities_filtered(&mut self) {
        let date_range = self.date_range;
        for entity in self.working_entities.iter_mut() {
            entity.update_filtered_by_bool_tag_expr(&self.entity_filter);
            entity.update_filtered_by_date_range(&date_range);
        }
    }

    /// Clamp the global offset
    pub fn clamp_global_offset(&mut self) {
        // Restrict top left from being dragged down and right from that point
        self.offset.x = self.offset.x.min(0.0);
        self.offset.y = self.offset.y.min(0.0);

        // Get max X and max Y points for entities
        let max_x_entity = self.working_entities.iter().max_by(|a, b| {
            a.max_x()
                .partial_cmp(&b.max_x())
                .unwrap_or(std::cmp::Ordering::Less)
        });
        let max_y_entity = self.working_entities.iter().max_by(|a, b| {
            a.max_y()
                .partial_cmp(&b.max_y())
                .unwrap_or(std::cmp::Ordering::Less)
        });

        // Update offsets
        if let Some(max_x_entity) = max_x_entity {
            let max_x = max_x_entity.max_x();
            let timeline_is_wider_than_canvas = max_x > self.canvas_size.x;
            if timeline_is_wider_than_canvas {
                self.offset.x = self.offset.x.max(self.canvas_size.x - max_x);
            } else {
                self.offset.x = self.offset.x.max(0.0);
            }
        }
        if let Some(max_y_entity) = max_y_entity {
            let max_y = max_y_entity.max_y() + self.zoomed_layout_params.row_margin;
            let timeline_is_taller_than_canvas = max_y > self.canvas_size.y;
            if timeline_is_taller_than_canvas {
                self.offset.y = self.offset.y.max(self.canvas_size.y - max_y);
            } else {
                self.offset.y = self.offset.y.max(0.0);
            }
        }
    }

    /// Zoom in around the mouse, by a factor of 1.1
    pub fn zoom_in(&mut self, mut factor: f64, x_local_offset: f64, y_local_offset: f64) {
        // Limit the maximum zoom.  Adjust the factor to avoid the timeline jumping
        // when max zoom reached.  Float comparison is fine because we only set it
        // here and we set it to an exact value
        if self.zoom == MAX_ZOOM {
            return;
        }
        if (self.zoom * factor) > MAX_ZOOM {
            factor = MAX_ZOOM / self.zoom;
            self.zoom = MAX_ZOOM;
        } else {
            self.zoom *= factor;
        }

        // Update the offset so that it appears as though we are zooming in
        // around the mouse
        self.offset.x = x_local_offset - ((x_local_offset - self.offset.x) * factor);
        self.offset.y = y_local_offset - ((y_local_offset - self.offset.y) * factor);

        // Update zoomed parameters
        self.update_zoomed_layout_params();

        // Recalculate: scaling doesn't linearly affect text sizing (though the
        // difference is small)
        self.re_calculate();
    }

    /// Zoom out around the mouse, by a factor of 1.1
    pub fn zoom_out(&mut self, mut factor: f64, x_local_offset: f64, y_local_offset: f64) {
        // Limit the minimum zoom.  Adjust the factor to avoid the timeline jumping
        // when max zoom reached.  Float comparison is fine because we only set it
        // here and we set it to an exact value
        if self.zoom == MIN_ZOOM {
            return;
        }
        if (self.zoom / factor) < MIN_ZOOM {
            factor = self.zoom / MIN_ZOOM;
            self.zoom = MIN_ZOOM;
        } else {
            self.zoom /= factor;
        }

        // Update the offset so that it appears as though we are zooming out
        // around the mouse
        self.offset.x = x_local_offset - ((x_local_offset - self.offset.x) / factor);
        self.offset.y = y_local_offset - ((y_local_offset - self.offset.y) / factor);

        // Update zoomed parameters
        self.update_zoomed_layout_params();

        // Recalculate: scaling doesn't linearly affect text sizing (though the
        // difference is small)
        self.re_calculate();
    }

    /// Set the zoom (use for jumping to zoom level).  Values are clamped
    /// between `MIN_SCALE` and `MAX_SCALE`
    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        self.update_zoomed_layout_params();
        self.re_calculate();
    }

    /// Set the datetime scale (for zooming into years and out to decades)
    pub fn set_datetime_scale(&mut self, scale: f64) {
        self.datetime_scale = scale.clamp(MIN_DATETIME_SCALE, MAX_DATETIME_SCALE);
        self.update_zoomed_layout_params();
        self.re_calculate();
    }

    /// Calculate the decade with using the measured year width (this accounts
    /// for padding)
    fn decade_width(&self) -> f64 {
        self.measured_layout_params.year_width * 10.0
    }

    // TODO: fix this so that we don't create everytime
    fn update_headings(&mut self) {
        let height = self.measured_layout_params.row_height_no_padding
            + (2.0 * self.zoomed_layout_params.padding_y);
        let decade_str_width = self.str_width("1234s");

        let mut headings = Vec::new();
        let mut current_decade = self.date_range.decade_range_start;
        for decade_number in 0..self.date_range.decade_count {
            let decade_number = f64::from(decade_number);
            let decade_string = format!("{current_decade}s");
            let decade_width = self.decade_width();
            let x = decade_width * decade_number;
            let text_x = x + (decade_width - decade_str_width) / 2.0;

            headings.push(Heading {
                text: TextOut {
                    top_left: Point {
                        x: text_x,
                        y: (self.zoomed_layout_params.padding_y),
                    },
                    text: decade_string,
                    colour: self.colours.heading.text_colour,
                    font_size: self.zoomed_layout_params.font_size_px,
                },
                text_box: FilledBox {
                    position_and_size: PositionAndSize {
                        position: Point { x, y: 0.0 },
                        width: decade_width,
                        height,
                    },
                    fill_colour: self.colours.heading.rect.fill_colour,
                    border_style: self.colours.heading.rect.border,
                },
            });

            // Years
            if self.datetime_scale() > DATETIME_SCALE_THRESHOLD_SHOW_YEARS {
                let year_width = self.decade_width() / 10.0;
                for year_number in 0..10 {
                    // Get the year value
                    let year = current_decade + year_number;

                    // Get the min x position
                    let x = x + (year_width * (year_number as f64));

                    // Derive the text string (e.g. '34 or 1234)
                    let text = if self.datetime_scale() < DATETIME_SCALE_THRESHOLD_SHOW_FULL_YEARS {
                        format!("'{:02}", year % 100)
                    } else {
                        format!("{year}")
                    };

                    // Calculate the text width & min x position
                    let text_width = self.str_width(&text);
                    let text_x = x + (year_width - text_width) / 2.0;

                    // Create the heading and add it to the list
                    headings.push(Heading {
                        text: TextOut {
                            top_left: Point {
                                x: text_x,
                                y: height + self.zoomed_layout_params.padding_y,
                            },
                            text,
                            colour: self.colours.heading.text_colour,
                            font_size: self.zoomed_layout_params.font_size_px,
                        },
                        text_box: FilledBox {
                            position_and_size: PositionAndSize {
                                position: Point { x, y: height },
                                width: year_width,
                                height,
                            },
                            fill_colour: self.colours.heading.rect.fill_colour,
                            border_style: self.colours.heading.rect.border,
                        },
                    });
                }
            }

            // Increment the decade
            current_decade += 10;
        }

        // Set the headings
        self.headings = headings;
    }

    // TODO: switch to using whole Date rather than just year
    /// Find and save the earliest (start) and latest (end) year
    fn update_earliest_and_latest_years(&mut self) {
        self.date_range.earliest_year = i32::MAX;
        self.date_range.latest_year = i32::MIN;

        for entity in &self.working_entities {
            // Ignore entities not shown
            if entity.is_filtered_out() {
                continue;
            }

            // Update earliest year
            self.date_range.earliest_year = self
                .date_range
                .earliest_year
                .min(entity.entity.start_year().value());

            // Update latest year
            if let Some(end_year) = entity.entity.end_year() {
                if end_year.value() > self.date_range.latest_year {
                    self.date_range.latest_year = end_year.value();
                }
            }
        }

        // Ensure the latest year isn't "unset"
        if self.date_range.latest_year == i32::MIN {
            self.date_range.latest_year = Year::current().value();
        }
    }

    /// Update timeline's date range value (largest, smallest, number of decades)
    fn update_timeline_date_range(&mut self) {
        self.update_earliest_and_latest_years();

        // TODO: do this more robustly
        if self.working_entities.is_empty() {
            self.date_range.decade_count = 0;
            return;
        }

        // Get the start year (the user defined cutoff if there is one,
        // otherwise the earliest entity start date)
        let start_year = {
            if let Some(start_date_cutoff) = self.date_range.start_date_cutoff {
                start_date_cutoff.year().value()
            } else {
                self.date_range.earliest_year
            }
        };

        // Get the end year (the user defined cutoff if there is one,
        // otherwise the latest entity end date)
        let end_year = {
            if let Some(end_date_cutoff) = self.date_range.end_date_cutoff {
                end_date_cutoff.year().value()
            } else {
                self.date_range.latest_year
            }
        };

        // Update the decade range values
        self.date_range.decade_range_start = floor_to_decade(start_year);
        self.date_range.decade_range_end = ceiling_to_decade(end_year);

        // Calculate the number of decades
        self.date_range.decade_count = (self
            .date_range
            .decade_range_end
            .saturating_sub(self.date_range.decade_range_start))
            / 10;
    }

    /// Sort entities by start year
    ///
    /// Cannot use a `.partial_cmp()` and declare them equal if none because
    /// this doesn't result in a total ordering.
    fn sort_entities(&mut self) {
        self.working_entities
            .sort_by(|a, b| a.entity.start().cmp(&b.entity.start()))
    }

    /// Put the working entities into place
    fn put_entities_in_rows(&mut self) {
        let mut rows: Vec<f64> = Vec::new();
        for entity in &mut self.working_entities {
            if entity.is_filtered_out() {
                continue;
            }
            let mut found_row = false;
            for (i, row_current_max_x) in rows.iter_mut().enumerate() {
                // Calculate the row's current max x value
                let row_max_x = {
                    let row_max_x = *row_current_max_x;
                    let row_max_x = row_max_x + self.zoomed_layout_params.min_inline_spacing;
                    round_f64_to_nearest_0_1(row_max_x)
                };

                // Calculate the entity's min x value
                let entity_min = round_f64_to_nearest_0_1(entity.min_x());

                // If the entity's min x value is greater than the row's current
                // max x value, then it is to the right of whatever is in the
                // row (the row may be empty).
                if row_max_x < entity_min {
                    // Update the row's current max x value to be equal to the
                    // entity's max x value
                    *row_current_max_x = entity.max_x();

                    // Give the entity a row number
                    entity.set_row(i);

                    // End the search for a row for this entity
                    found_row = true;
                    break;
                }
            }
            // If the entity doesn't fit into any of the rows, create a new one
            // for it
            if !found_row {
                entity.set_row(rows.len());
                rows.push(entity.max_x());
            }
        }
    }

    /// Put the working entities into place
    fn calculate_entity_positions(&mut self) {
        // Calculate the width and x position of every entity so that we can put
        // them into rows (we need to know their max x values)
        self.calculate_widths_for_entities();
        self.calculate_x_position_for_entities();
        self.put_entities_in_rows();

        // Calculate y position of every entity now that we know what row it is
        // in
        self.calculate_y_position_for_entities();

        // Update the global offset
        self.clamp_global_offset();
    }

    /// Calculate each entity's width.  This must be done so that entities can
    /// be put into rows
    fn calculate_widths_for_entities(&mut self) {
        for entity in &mut self.working_entities {
            // Get the end year of the entity, or the last year of the timeline
            // if it doesn't have one
            let end_year = entity
                .entity
                .end_year()
                .unwrap_or(Year::try_from(self.date_range.decade_range_end as i64).unwrap());

            let start_month_and_day_as_fraction_of_year = month_and_day_as_fraction_of_year(
                entity.entity.start_month(),
                entity.entity.start_day(),
            );
            let end_month_and_day_as_fraction_of_year = month_and_day_as_fraction_of_year(
                entity.entity.end_month(),
                entity.entity.end_day(),
            );

            // Get the entity's lifespan (years)
            let entity_number_of_years = (end_year.value() as f64
                + end_month_and_day_as_fraction_of_year)
                - (entity.entity.start_year().value() as f64
                    + start_month_and_day_as_fraction_of_year);

            // Calculate the entity's date box width using it's lifespan
            let date_box_width = (entity_number_of_years) * self.measured_layout_params.year_width;
            entity.date_box.position_and_size.width = date_box_width;

            // Calculate the entity's text box width using the width of it's name text
            let text_box_width = entity.text.width + (2.0 * self.zoomed_layout_params.padding_x);
            entity.text_box.position_and_size.width = text_box_width;
        }
    }

    /// Calculate each entity's x position.
    ///
    /// Global offset is added later just before the entities are returned for
    /// drawing.
    fn calculate_x_position_for_entities(&mut self) {
        for entity in &mut self.working_entities {
            // Calculate the number of years between the timeline's start year and
            // the start year of the entity
            let offset_in_years =
                entity.entity.start_year().value() - self.date_range.decade_range_start;

            // Calculate the x position of the entity
            let start_month = entity.entity.start_month();
            let start_day = entity.entity.start_day();
            let x: f64 = ((offset_in_years as f64)
                + month_and_day_as_fraction_of_year(start_month, start_day))
                * self.measured_layout_params.year_width;

            // Set the x positions
            entity.text.top_left.x = x + self.zoomed_layout_params.padding_x;
            entity.text_box.position_and_size.position.x = x;
            entity.date_box.position_and_size.position.x = x;
        }
    }

    /// Calculate each entity's y position.  All entities must have a row number
    /// before this function is called.
    ///
    /// Global offset is added later just before the entities are returned for
    /// drawing.
    fn calculate_y_position_for_entities(&mut self) {
        for entity in &mut self.working_entities {
            // Calculate the actual row height (add in padding and margin)
            let row_height = self.measured_layout_params.row_height_no_padding
                + self.zoomed_layout_params.row_margin
                + (self.zoomed_layout_params.padding_y * 2.0);

            // Calculate the y position using the row height and the entity's row
            let y = row_height * ((entity.row() + 1) as f64);

            // Set the y positions
            entity.text.top_left.y = y + self.zoomed_layout_params.padding_y;
            entity.text_box.position_and_size.position.y = y;
            entity.date_box.position_and_size.position.y = y;
        }
    }
}

fn month_and_day_as_fraction_of_year(month: Option<Month>, day: Option<Day>) -> f64 {
    let month_number = month.map_or(1, |month| month.value()) - 1;
    let day_number = day.map_or(1, |day| day.value()) - 1;
    (month_number as f64 / 12.0) + (day_number as f64 / 365.0)
}

/// Calculate whether the thing is visible on the canvas
fn is_visible(thing_min: Point, thing_max: Point, canvas_size: Point) -> bool {
    let height = thing_max.y - thing_min.y;
    if thing_min.x > canvas_size.x {
        return false;
    }
    if thing_max.x < 0.0 {
        return false;
    }
    if thing_min.y - height > canvas_size.y {
        return false;
    }
    if thing_max.y + height < 0.0 {
        return false;
    }
    true
}
