// SPDX-License-Identifier: MIT

//!
//! The egui frontend
//!

// TODO: add a pass through macro (for use by both frontends) so that a method
// on `Engine` is exposed in the same way for the front end - can we even reuse
// the doc comment?

use crate::{Colour, Engine, PositionAndSize, TimelineColours, TimelineInteractionEvent};
use bool_tag_expr::BoolTagExpr;
use eframe::egui::{
    Align2, Color32, Context, FontId, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use log::*;
use open_timeline_core::{Date, Entity, HasIdAndName};

/// The HTML canvas engine for use on the web
pub struct OpenTimelineRendererEgui {
    /// The underlying timeline [`Engine`].
    engine: Engine,
}

impl OpenTimelineRendererEgui {
    /// Create a new HTML canvas engine
    pub fn new(ctx: &Context) -> Self {
        info!("Constructing a new EguiRenderer in Rust");
        let ctx_clone = ctx.clone();

        // TODO: do this with a method?
        let text_measurer =
            move |font_size, text| measure_text_fn(ctx_clone.clone(), font_size, text);
        Self {
            engine: Engine::new(text_measurer),
        }
    }

    pub fn clear_entities(&mut self, ctx: &Context, ui: &mut Ui) {
        self.engine.clear_entities();

        self.draw(ctx, ui);
        // debug!("redrawn with no entities");
    }

    pub fn set_entities(&mut self, entities: Vec<Entity>) {
        self.engine.set_entities(entities);
    }

    pub fn add_entities(&mut self, entities: Vec<Entity>) {
        // debug!("add_entities");

        self.engine.add_entities(entities);
        // debug!("added entities to the engine");
    }

    // TODO: redraw? This is inconsistent across frontends (I think)
    pub fn set_font_size_px(&mut self, font_size: f64) {
        self.engine.set_font_size_px(font_size);
        // debug!("redrawn with new font size");
    }

    pub fn start_and_end_dates(&mut self) -> (i32, i32) {
        self.engine.start_and_end_dates()
    }

    pub fn zoom_in(&mut self, factor: f64, x_local_offset: f64, y_local_offset: f64) {
        self.engine.zoom_in(factor, x_local_offset, y_local_offset);
    }

    pub fn zoom_out(&mut self, factor: f64, x_local_offset: f64, y_local_offset: f64) {
        self.engine.zoom_out(factor, x_local_offset, y_local_offset);
    }

    pub fn set_tag_bool_expr_entity_filter(&mut self, tag_bool_expr: BoolTagExpr) {
        self.engine.set_tag_bool_expr_entity_filter(tag_bool_expr);
    }

    pub fn remove_tag_bool_expr_entity_filter(&mut self) {
        self.engine.remove_tag_bool_expr_entity_filter();
    }

    pub fn set_date_limits(&mut self, start: Option<Date>, end: Option<Date>) {
        self.engine.set_date_limits(start, end);
    }

    pub fn drain_interaction_events(&mut self) -> std::vec::Drain<'_, TimelineInteractionEvent> {
        self.engine.drain_interaction_events()
    }

    pub fn entity_count(&mut self) -> usize {
        self.engine.entity_count()
    }

    pub fn set_sticky_text(&mut self, sticky_text: bool) {
        self.engine.set_sticky_text(sticky_text)
    }

    pub fn set_datetime_scale(&mut self, scale: f64) {
        self.engine.set_datetime_scale(scale)
    }

    pub fn draw(&mut self, ctx: &Context, ui: &mut Ui) {
        draw_timeline(ctx, ui, &mut self.engine);
        // debug!("[exit] .draw()");
    }

    pub fn colours(&mut self) -> TimelineColours {
        self.engine.colours()
    }

    // TODO: this is horrible (trying not to log every frame). Fix check_for_timeline_colour_changes()
    // which is using blocking_read() (remove all blocking_read()s)
    pub fn set_colours(&mut self, colours: TimelineColours) {
        if colours != self.engine.colours() {
            debug!("egui renderer set colours");
            self.engine.set_colours(colours)
        }
    }
}

/// Function supplied to the [`Engine`] so that it can measure text (used in its
/// calculations)
fn measure_text_fn(ctx: Context, font_size: f64, text: String) -> (f64, f64) {
    let text_galley = ctx.fonts_mut(|f| {
        f.layout_no_wrap(text, FontId::proportional(font_size as f32), Color32::BLACK)
    });
    let text_width: f64 = text_galley.rect.width().into();
    let text_height: f64 = text_galley.rect.height().into();
    (text_width, text_height)
}

/// Draw the timeline in an `egui` application
fn draw_timeline(_ctx: &Context, ui: &mut Ui, engine: &mut Engine) {
    let width = ui.available_width();
    let height = ui.available_height();
    let (painter_response, painter) = ui.allocate_painter(Vec2::new(width, height), Sense::drag());

    // Move the timeline if the user is dragging it
    if painter_response.dragged() {
        let delta = painter_response.drag_motion();
        engine.add_to_global_offset(delta.x.into(), delta.y.into());
    }

    let canvas_rect = painter_response.rect;
    let canvas_min = canvas_rect.min.to_vec2();
    let canvas_max = canvas_rect.max.to_vec2();
    let canvas_size = canvas_max - canvas_min;
    engine.set_canvas_max(canvas_size.x.into(), canvas_size.y.into());

    // Draw background stripes
    for background in engine.backgrounds_for_drawing() {
        let (r, g, b) = background.colour.as_rgb();
        let min = Pos2::new(background.x as f32, 0.0);
        let max = Pos2::new(background.x as f32 + background.width as f32, f32::MAX);
        let rect = Rect::from_two_pos(min + canvas_min, max + canvas_min);
        painter.rect(
            rect,
            0.0,
            Color32::from_rgb(r, g, b),
            Stroke::NONE,
            StrokeKind::Inside,
        );
    }

    // Draw lines
    let clip = painter.clip_rect();
    let top_y = clip.top();
    let bottom_y = clip.bottom();
    for line in engine.lines_for_drawing() {
        painter.vline(
            canvas_min.x + line.x as f32,
            top_y..=bottom_y,
            Stroke::new(
                line.style.thickness as f32,
                timeline_renderer_colour_to_egui_colour(line.style.colour),
            ),
        );
    }

    let mut hovering_over_entities = false;

    // TODO: can still click & hover over entities under the headings (fix in engine)
    // Draw entities
    for entity in engine.entities_for_drawing() {
        // Draw text box
        let text_box = &entity.text_box;
        let (min, max) = timeline_renderer_position_and_size_to_min_and_max_egui_pos2(
            &text_box.position_and_size,
        );
        let text_box_rect = Rect::from_two_pos(min + canvas_min, max + canvas_min);
        let (thickness, colour) = {
            if let Some(border_style) = text_box.border_style {
                (border_style.thickness, border_style.colour)
            } else {
                (0.0, Colour::from_rgb(0, 0, 0))
            }
        };
        painter.rect(
            text_box_rect,
            0.0,
            timeline_renderer_colour_to_egui_colour(text_box.fill_colour),
            Stroke::new(thickness as f32, colour),
            StrokeKind::Inside,
        );

        // Draw date box
        let date_box = &entity.date_box;
        let (min, max) = timeline_renderer_position_and_size_to_min_and_max_egui_pos2(
            &date_box.position_and_size,
        );
        let date_box_rect = Rect::from_two_pos(min + canvas_min, max + canvas_min);
        let (thickness, colour) = {
            if let Some(border_style) = date_box.border_style {
                (border_style.thickness, border_style.colour)
            } else {
                (0.0, Colour::from_rgb(0, 0, 0))
            }
        };
        painter.rect(
            date_box_rect,
            0.0,
            timeline_renderer_colour_to_egui_colour(date_box.fill_colour),
            Stroke::new(thickness as f32, colour),
            StrokeKind::Inside,
        );

        // Don't sense clicking on things outside the canvas.  Without the
        // `.intersect()` with the canvas rect, one could move the timeline and
        // then click on one of the control buttons, only to have a timeline
        // entity view window to pop open
        let bounding_rect = date_box_rect.union(text_box_rect);
        let visible_rect = painter_response.rect.intersect(bounding_rect);
        let entity_response = ui.interact(
            visible_rect,
            Id::from(entity.entity.id().unwrap().to_string()),
            Sense::click(),
        );

        // Hover over entity
        if entity_response.hovered() {
            hovering_over_entities = true;
            engine.hover_over_entity(Some(entity.entity.id().unwrap()));
        }

        // Click on entity
        if entity_response.clicked() {
            if let Some(entity_id) = entity.entity.id() {
                engine.click_on_entity(entity_id);
            }
        }

        // Write text
        let text = &entity.text;
        let pos = Pos2::new(text.top_left.x as f32, text.top_left.y as f32);
        painter.text(
            pos + canvas_min,
            Align2::LEFT_TOP,
            &text.text,
            FontId::proportional(text.font_size as f32),
            timeline_renderer_colour_to_egui_colour(text.colour),
        );
    }

    //
    if !hovering_over_entities {
        engine.hover_over_entity(None);
    }

    // Draw headings
    for heading in engine.headings_for_drawing() {
        let text_box = &heading.text_box;
        let (min, max) = timeline_renderer_position_and_size_to_min_and_max_egui_pos2(
            &text_box.position_and_size,
        );
        let rect = Rect::from_two_pos(min + canvas_min, max + canvas_min);
        let (thickness, colour) = {
            if let Some(border_style) = text_box.border_style {
                (border_style.thickness, border_style.colour)
            } else {
                (0.0, Colour::from_rgb(0, 0, 0))
            }
        };
        painter.rect(
            rect,
            0.0,
            timeline_renderer_colour_to_egui_colour(text_box.fill_colour),
            Stroke::new(thickness as f32, colour),
            StrokeKind::Inside,
        );

        // Write text
        let text = &heading.text;
        let pos = Pos2::new(text.top_left.x as f32, text.top_left.y as f32);
        painter.text(
            pos + canvas_min,
            Align2::LEFT_TOP,
            &text.text,
            FontId::proportional(text.font_size as f32),
            timeline_renderer_colour_to_egui_colour(text.colour),
        );
    }

    // TODO
    // let stroke = Stroke::new(1.0, Color32::LIGHT_RED);
    let stroke = Stroke::NONE;

    // Draw timeline border
    painter.rect_stroke(canvas_rect, 0.0, stroke, StrokeKind::Inside);

    // Handle any scrolling & zooming input
    if painter_response.hovered() {
        let (x_scroll, y_scroll) = ui.input(|i| (i.smooth_scroll_delta.x, i.smooth_scroll_delta.y));
        let zoom_delta = ui.input(|i| i.zoom_delta());
        if zoom_delta > 1.01 {
            engine.zoom_in(zoom_delta.into(), 0.0, 0.0);
        } else if zoom_delta < 0.99 {
            engine.zoom_out((1.0 / zoom_delta).into(), 0.0, 0.0);
        }
        engine.add_to_global_offset(x_scroll.into(), y_scroll.into());
    }
}

// TODO: move these
/// Convert a [`Colour`] to a [`Color32`]
fn timeline_renderer_colour_to_egui_colour(colour: Colour) -> Color32 {
    let (r, g, b) = colour.as_rgb();
    Color32::from_rgb(r, g, b)
}

// TODO: impl Into?
/// Convert a [`PositionAndSize`] into min and max [`Pos2`]s
fn timeline_renderer_position_and_size_to_min_and_max_egui_pos2(
    position_and_size: &PositionAndSize,
) -> (Pos2, Pos2) {
    let min = Pos2::new(
        position_and_size.position.x as f32,
        position_and_size.position.y as f32,
    );

    // TODO: should use max_x() and max_y() methods (but need padding)
    let max = Pos2::new(
        position_and_size.position.x as f32 + position_and_size.width as f32,
        position_and_size.position.y as f32 + position_and_size.height as f32,
    );
    (min, max)
}
