// SPDX-License-Identifier: MIT

//!
//! The HTML Canvas frontend
//!
//! ```sh
//! wasm-pack build --target web
//! python3 -m http.server 8000 --bind 0.0.0.0
//! ```
//!

use crate::{Colour, Engine, FilledBox, Position, ScalableLayoutParams, TextOut};
use chrono::Local;
use log::{debug, info};
use open_timeline_core::{Entity, HasIdAndName, OpenTimelineId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::{Closure, wasm_bindgen};
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, KeyboardEvent, MouseEvent,
    TextMetrics, TouchEvent, WheelEvent,
};

// TODO
// - Use unwrap_throw() more (see what it does first)
// - Ability to pass in CSS query selectors/other

// Setup from JS object (have a reference one to compare with)
// for field in setup_dict {
//     if self.hasOwnProperty(field) {
//         self[field] = setup_dict[field]
//     }
// }

// //------------------------------------------------------------------------------
// // Convert a valid CSS colour to RGB
// //
// // Work
// // - green
// // - 008000
// // - #008000
// //
// // Don't work
// // - greesdfn
// // - 008rra
// //------------------------------------------------------------------------------
// function cssColourToRgb(colour) {
//     // Create a temporary element (has height 0)
//     let tmp_el = document.createElement("div")
//     tmp_el.style.color = colour
//     tmp_el.style.position = "absolute"
//     tmp_el.style.left = "-9999px"
//     tmp_el.style.top = "-9999px"
//     document.body.appendChild(tmp_el)
//     let rgb = window.getComputedStyle(tmp_el).color
//     document.body.removeChild(tmp_el)
//     let matches = rgb.match(/^rgb\((\d+),\s*(\d+),\s*(\d+)\)$/)
//     if (matches.length != 4) {
//         return false
//     }
//     let [_, r_str, g_str, b_str] = matches
//     let r = Number(r_str)
//     let g = Number(g_str)
//     let b = Number(b_str)
//     return {r, g, b}
// }

/// Function supplied to the [`Engine`] so that it can measure text (used in its
/// calculations)
pub fn measure_text(font_size: f64, text: &str) -> TextMetrics {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .query_selector("canvas[visible]")
        .unwrap()
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    let font_size = font_size * device_pixel_ratio();

    let font_style = "serif";
    ctx.set_font(&format!("{font_size}px {font_style}"));

    ctx.measure_text(&text).unwrap()
}

// TODO: use to allow for styling API?
///
///
/// # Usage
///
/// generate_simple_get_and_set!(start: Date);
macro_rules! _generate_simple_get_and_set {
    ( $field_name:ident: $arg_type:ty) => {
        paste! {
            pub fn [<get_ $field_name>](&self) -> $arg_type {
                self.$field_name.clone()
            }
        }
        paste! {
            pub fn [<set_ $field_name>](&mut self, $field_name: $arg_type) {
                self.$field_name = $field_name
            }
        }
    };
}

/// Function supplied to the [`Engine`] so that it can measure text (used in its
/// calculations)
pub fn measure_text_for_engine(font_size: f64, text: String) -> (f64, f64) {
    let measurements = measure_text(font_size, &text);
    let height =
        measurements.actual_bounding_box_ascent() + measurements.actual_bounding_box_descent();
    (measurements.width(), height)
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // TODO: where to put this?
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    log::info!("Start OpenTimeline");
    Ok(())
}

#[derive(Debug)]
struct State {
    // TODO: move this?
    /// Maps the colours on the hidden canvas to an entity's ID
    map: HashMap<Colour, OpenTimelineId>,

    /// x & y coordinates of where on the canvas the user is touching (if they
    /// are)
    touch_position: Option<Position>,

    /// The timestamp (ms) of the last tap
    time_of_last_tap: i64,

    /// The timestamp (ms) of the last double tap
    time_of_last_double_tap: i64,

    /// The timestamp (ms) of the last triple tap
    time_of_last_triple_tap: i64,

    /// The ID associated with the most recent touchstart event.
    ///
    /// This is used to correctly distinguish single/double/triple taps
    most_recent_touchstart_event_id: i64,

    /// Whether the timeline if being dragged (i.e. moved).
    dragging: bool,

    /// Is the mouse click button currently held down
    mouse_is_down: bool,

    /// When was the user last draggin (i.e. moving the timeline)
    ms_time_of_last_dragging: i64,
}

/// Whether the event is to target the visible canvas or the window
#[derive(Debug, Copy, Clone)]
enum EventListenTarget {
    VisibleCanvas,
    Window,
}

/// A canvas and associated context
#[derive(Debug, Clone)]
struct CanvasAndContext {
    /// The canvas element
    canvas: HtmlCanvasElement,

    /// The context for the associated canvas
    ctx: CanvasRenderingContext2d,
}

#[derive(Debug, Clone)]
struct DrawingSurfaces {
    /// The visible canvas (and context)
    visible: CanvasAndContext,

    /// The invisible canvas (and context)
    invisible: CanvasAndContext,
}

impl DrawingSurfaces {
    fn for_demo() -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let demo_drawing_surfaces: Vec<CanvasAndContext> = ["canvas[visible]", "canvas[invisible]"]
            .into_iter()
            .map(|selector| {
                let canvas = document
                    .query_selector(selector)
                    .unwrap()
                    .unwrap()
                    .dyn_into::<HtmlCanvasElement>()
                    .unwrap();
                let ctx = canvas
                    .get_context("2d")
                    .unwrap_throw()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()
                    .unwrap_throw();
                CanvasAndContext { canvas, ctx }
            })
            .collect();

        // debug!("demo_drawing_surfaces = {demo_drawing_surfaces:?}");

        Self {
            visible: demo_drawing_surfaces[0].clone(),
            invisible: demo_drawing_surfaces[1].clone(),
        }
    }
}

// Draw order is: clear canvas, draw background, draw entities, draw headings (had to do full thing twice before)

// TODO: Add set() for entities so that we can update the Engine automatically
// TODO: Does it matter if these are public or private?
/// The HTML canvas engine for use on the web
#[wasm_bindgen]
pub struct OpenTimelineRendererHtmlCanvas {
    /// The underlying timeline [`Engine`].
    engine: Rc<RefCell<Engine>>,

    ///
    state: Rc<RefCell<State>>,

    ///
    drawing_surfaces: Rc<RefCell<DrawingSurfaces>>,
}

#[wasm_bindgen]
impl OpenTimelineRendererHtmlCanvas {
    //--------------------------------------------------------------------------
    // WASM bindgen functions
    //--------------------------------------------------------------------------

    /// Create a new HTML canvas engine
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        info!("Constructing a new HtmlCanvas in Rust");

        // TODO: do this with a method?
        let mut engine = Engine::new(measure_text_for_engine);

        // TODO
        engine.set_canvas_max(600.0 * device_pixel_ratio(), 400.0 * device_pixel_ratio());

        engine.set_font_size_px(engine.effective_font_size_px() * 1.5);
        engine.set_layout_params(ScalableLayoutParams {
            row_margin: 10.0,
            min_inline_spacing: 7.5,
            padding_x: 20.0,
            padding_y: 25.0,
            font_size_px: 14.0,
            dividing_line_thickness: 0.5,
            entity_highlight_thickness: 10.0,
        });

        //
        let mut html_canvas = Self {
            drawing_surfaces: Rc::new(RefCell::new(DrawingSurfaces::for_demo())),
            state: Rc::new(RefCell::new(State {
                map: HashMap::new(),
                touch_position: None,
                time_of_last_tap: 0,
                time_of_last_double_tap: 0,
                time_of_last_triple_tap: 0,
                most_recent_touchstart_event_id: 0,
                dragging: false,
                mouse_is_down: false,
                ms_time_of_last_dragging: Local::now().timestamp_millis(),
            })),
            engine: Rc::new(RefCell::new(engine)),
        };

        info!("Setting up listeneres");
        html_canvas.listen_for_mousedown();
        html_canvas.listen_for_mouseup();
        html_canvas.listen_for_mousemove();
        html_canvas.listen_for_mouseleave();
        html_canvas.listen_for_click();
        html_canvas.listen_for_scroll();
        html_canvas.listen_for_touchstart();
        html_canvas.listen_for_touchmove();
        html_canvas.listen_for_touchend();
        html_canvas.listen_for_keydown();
        html_canvas
    }

    #[wasm_bindgen]
    pub fn clear_entities(&mut self) {
        // debug!("clear_entities");
        self.engine.borrow_mut().clear_entities();
    }

    #[wasm_bindgen]
    pub fn set_entities(&mut self, entities: JsValue) -> Result<(), JsValue> {
        // debug!("set_entities");
        self.clear_entities();
        self.add_entities(entities)
    }

    #[wasm_bindgen]
    pub fn add_entities(&mut self, entities: JsValue) -> Result<(), JsValue> {
        // debug!("add_entities");
        let entities: Vec<Entity> = serde_wasm_bindgen::from_value(entities).unwrap();
        // debug!("got vec of entities");

        for entity in &entities {
            self.state.borrow_mut().map.insert(
                Colour::from_any_string(entity.name().as_str()),
                entity.id().unwrap(),
            );
        }
        // debug!("added hidden colours for entities");

        self.engine.borrow_mut().add_entities(entities);
        // debug!("added entities to the engine");
        self.draw();
        // debug!("redrawn with new entities");
        Ok(())
    }

    //--------------------------------------------------------------------------
    //
    //--------------------------------------------------------------------------
    #[wasm_bindgen]
    pub fn draw(&mut self) {
        draw_timeline(self.engine.clone(), self.drawing_surfaces.clone());
        // debug!("[exit] .draw()");
    }

    //--------------------------------------------------------------------------
    // Manage events
    //--------------------------------------------------------------------------

    // TODO: I previous was trying to save the event listeners in a vec, but was encountering
    // recursion/called after dropped error so switched to .forget()
    fn add_listener<E, F>(&mut self, target: EventListenTarget, event_name: &str, mut listener: F)
    where
        E: JsCast + 'static,
        F: FnMut(E) + 'static,
    {
        // let event_name_string = event_name.to_string();
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            if let Ok(event) = event.dyn_into::<E>() {
                listener(event);
                // debug!("Returned from event handler for {event_name_string}");
            }
            // debug!("Exiting event handler closure");
        }) as Box<dyn FnMut(web_sys::Event)>);

        // Attach the closure to the canvas
        match target {
            EventListenTarget::VisibleCanvas => self
                .drawing_surfaces
                .borrow()
                .visible
                .canvas
                .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())
                .unwrap_throw(),
            EventListenTarget::Window => web_sys::window()
                .unwrap()
                .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())
                .unwrap_throw(),
        }

        // Keep the closure
        closure.forget();
    }

    /// Mousedown event handler
    pub fn listen_for_mousedown(&mut self) {
        let state_clone = self.state.clone();
        self.add_listener::<web_sys::MouseEvent, _>(
            EventListenTarget::VisibleCanvas,
            "mousedown",
            move |_event: MouseEvent| {
                // debug!("Mousedown event");
                state_clone.borrow_mut().mouse_is_down = true;
            },
        );
    }

    // TODO: identical to mouseleave
    /// Mouseup event handler
    pub fn listen_for_mouseup(&mut self) {
        let state = self.state.clone();
        self.add_listener::<web_sys::MouseEvent, _>(
            EventListenTarget::VisibleCanvas,
            "mouseup",
            move |_event: MouseEvent| {
                if state.borrow().dragging {
                    // debug!("Before Local::now()");
                    state.borrow_mut().ms_time_of_last_dragging = Local::now().timestamp_millis();
                    // debug!("After Local::now()");
                }
                state.borrow_mut().dragging = false;
                state.borrow_mut().mouse_is_down = false;
                // draw_timeline(engine.clone(), drawing_surfaces.clone());
            },
        );
    }

    /// Mouseout event handler
    pub fn listen_for_mouseleave(&mut self) {
        let state = self.state.clone();
        self.add_listener::<web_sys::MouseEvent, _>(
            EventListenTarget::VisibleCanvas,
            "mouseleave",
            move |_event: MouseEvent| {
                if state.borrow().dragging {
                    // debug!("Before Local::now()");
                    state.borrow_mut().ms_time_of_last_dragging = Local::now().timestamp_millis();
                    // debug!("After Local::now()");
                }
                state.borrow_mut().dragging = false;
                state.borrow_mut().mouse_is_down = false;
                // draw_timeline(engine.clone(), drawing_surfaces.clone());
            },
        );
    }

    /// Mousemove event
    ///
    /// Assume the mouse is over the timeline
    pub fn listen_for_mousemove(&mut self) {
        let drawing_surfaces = self.drawing_surfaces.clone();
        let engine = self.engine.clone();
        let state = self.state.clone();
        self.add_listener::<web_sys::MouseEvent, _>(
            EventListenTarget::VisibleCanvas,
            "mousemove",
            move |event: MouseEvent| {
                // info!("mousemove");

                // Get entity ID under mouse (hover over entity)
                let x = event.offset_x() as f64 * device_pixel_ratio();
                let y = event.offset_y() as f64 * device_pixel_ratio();
                if let Ok(colour_under_pointer) = colour_at_point(&drawing_surfaces, x, y) {
                    if let Some(id) = state.borrow().map.get(&colour_under_pointer) {
                        debug!("Hovering over: {id:?}");
                        engine.borrow_mut().hover_over_entity(Some(*id));
                    } else {
                        engine.borrow_mut().hover_over_entity(None);
                    }
                }

                // Update the global offset if dragging
                if state.borrow().mouse_is_down {
                    state.borrow_mut().dragging = true;
                    let del_x = event.movement_x() as f64 * device_pixel_ratio();
                    let del_y = event.movement_y() as f64 * device_pixel_ratio();
                    // debug!("Mousemove: ({}, {})", del_x, del_y);
                    engine.borrow_mut().add_to_global_offset(del_x, del_y);
                }

                // Draw
                // draw_timeline(engine.clone(), drawing_surfaces.clone());
            },
        );
    }

    /// Touch start event handler
    ///
    /// Double tap to zoom in, triple tap to zoom out
    pub fn listen_for_touchstart(&mut self) {
        let engine = self.engine.clone();
        let state = self.state.clone();
        self.add_listener::<web_sys::TouchEvent, _>(
            EventListenTarget::VisibleCanvas,
            "touchstart",
            move |event: TouchEvent| {
                // info!("touchstart");

                // Stop zooming in/out the page when double/triple tapping
                event.stop_propagation();
                event.prevent_default();

                // Set touch coords
                let touch = event.touches().get(0).unwrap();
                let current_x = touch.client_x() as f64;
                let current_y = touch.client_y() as f64;
                state.borrow_mut().touch_position = Some(Position {
                    x: current_x,
                    y: current_y,
                });

                // Use the current time in ms as an event ID
                let event_id = Local::now().timestamp_millis();
                state.borrow_mut().most_recent_touchstart_event_id = event_id;

                // Set the cutoff limit
                let cutoff_ms = 250;

                // Calculate the times since the last tap and last double tap
                let now = Local::now().timestamp_millis();
                let time_since_last_tap = now - state.borrow().time_of_last_tap;
                let time_since_last_double_tap = now - state.borrow().time_of_last_double_tap;
                let time_since_last_triple_tap = now - state.borrow().time_of_last_triple_tap;

                // Update the time of the last tap (ie this one)
                state.borrow_mut().time_of_last_tap = now;

                // Ignore anything more than a triple tap
                if time_since_last_triple_tap < cutoff_ms {
                    return;
                }

                // First check for a triple tap
                if time_since_last_double_tap < cutoff_ms {
                    info!("Triple tap detected");
                    state.borrow_mut().time_of_last_triple_tap = now;
                    engine.borrow_mut().zoom_out(1.5, 0.0, 0.0);
                    // draw_timeline(engine.clone(), drawing_surfaces.clone());
                    return;
                }

                // Check for a double tap (can't be a triple tap)
                if time_since_last_tap < cutoff_ms {
                    state.borrow_mut().time_of_last_double_tap = now;
                }

                // Shadow cutoff_ms with new integer type
                let cutoff_ms = 250;

                // Wait 250ms to check a double tap isn't a triple tap
                let engine = engine.clone();
                let state = state.clone();
                gloo_timers::callback::Timeout::new(cutoff_ms, move || {
                    // Shadow cutoff_ms with new integer type
                    let cutoff_ms = 250;

                    // If the latest event ID is this one then:
                    // - a double tap can't become a triple tap in the next event
                    // - a single tap can't become a single tap in the next event
                    if state.borrow().most_recent_touchstart_event_id == event_id {
                        if time_since_last_tap < cutoff_ms {
                            // Check for a double tap first
                            info!("Double tap detected");
                            engine.borrow_mut().zoom_in(1.5, 0.0, 0.0);
                            // draw_timeline(engine.clone(), drawing_surfaces.clone());
                        } else {
                            // Must be a single tap
                            info!("Single tap detected");
                        }
                    }
                })
                .forget();
                return;
            },
        );
    }

    /// Touch move event handler
    pub fn listen_for_touchmove(&mut self) {
        let engine = self.engine.clone();
        let state = self.state.clone();
        self.add_listener::<web_sys::TouchEvent, _>(
            EventListenTarget::VisibleCanvas,
            "touchmove",
            move |event: TouchEvent| {
                // info!("touchmove");
                let mut state = state.borrow_mut();
                event.stop_propagation();
                event.prevent_default();
                if let (Some(touch_position), Some(touch)) =
                    (state.touch_position, event.touches().get(0))
                {
                    let current_x = touch.client_x() as f64;
                    let current_y = touch.client_y() as f64;

                    let del_x = (touch_position.x - current_x) * device_pixel_ratio();
                    let del_y = (touch_position.y - current_y) * device_pixel_ratio();

                    debug!("{:?} {:?}", del_x, del_y);

                    engine.borrow_mut().add_to_global_offset(-del_x, -del_y);
                    state.touch_position = Some(Position {
                        x: current_x,
                        y: current_y,
                    });

                    // draw_timeline(engine.clone(), drawing_surfaces.clone());
                }
            },
        );
    }

    /// Touch end event handler
    pub fn listen_for_touchend(&mut self) {
        let state = self.state.clone();
        self.add_listener::<web_sys::TouchEvent, _>(
            EventListenTarget::VisibleCanvas,
            "touchend",
            move |_event: TouchEvent| {
                // info!("touchend");
                let mut state = state.borrow_mut();
                state.dragging = false;
                state.touch_position = None;
            },
        );
    }

    /// Scroll event handler
    pub fn listen_for_scroll(&mut self) {
        let engine = self.engine.clone();
        self.add_listener::<web_sys::WheelEvent, _>(
            EventListenTarget::VisibleCanvas,
            "wheel",
            move |event: WheelEvent| {
                // info!("wheel");
                event.stop_propagation();
                event.prevent_default();

                let scroll_to_zoom = event.ctrl_key() || event.meta_key();
                if scroll_to_zoom {
                    // Divide to reduce zoom speed.  Add 1 to ensure the factor
                    // is always greater than 1.
                    let factor = (event.delta_y().abs() / 250.0) + 1.0;
                    // debug!("Scroll zoom factor = {factor}");

                    // Get the position of the mouse
                    let x = event.offset_x().into();
                    let y = event.offset_y().into();

                    // Whether to zoom in or out
                    if event.delta_y() > 0.0 {
                        engine.borrow_mut().zoom_out(factor, x, y);
                    } else {
                        engine.borrow_mut().zoom_in(factor, x, y);
                    }
                } else {
                    engine
                        .borrow_mut()
                        .add_to_global_offset(-event.delta_x(), -event.delta_y());
                }
                // draw_timeline(engine.clone(), drawing_surfaces.clone());
            },
        );
    }

    // TODO: add double & triple click detection (already have double & triple tap detection)
    /// Manage a click event on the timeline
    ///
    /// Emit a custom event so that user can make use of the new
    /// selection (eg fill a form)
    pub fn listen_for_click(&mut self) {
        let drawing_surfaces = self.drawing_surfaces.clone();
        let engine = self.engine.clone();
        let state = self.state.clone();
        self.add_listener::<web_sys::MouseEvent, _>(
            EventListenTarget::VisibleCanvas,
            "click",
            move |event: MouseEvent| {
                // info!("click");
                let x = event.offset_x() as f64 * device_pixel_ratio();
                let y = event.offset_y() as f64 * device_pixel_ratio();
                if let Ok(colour_under_pointer) = colour_at_point(&drawing_surfaces, x, y) {
                    if let Some(id) = state.borrow().map.get(&colour_under_pointer) {
                        debug!("Clicked on: {id:?}");
                        engine.borrow_mut().click_on_entity(*id);
                    }
                }
            },
        );
    }

    /// Manage a keydown event
    pub fn listen_for_keydown(&mut self) {
        let drawing_surfaces = self.drawing_surfaces.clone();
        self.add_listener::<web_sys::KeyboardEvent, _>(
            EventListenTarget::Window,
            "keydown",
            move |event: KeyboardEvent| {
                // info!("keydown");

                // Ignore repeats
                if event.repeat() {
                    return;
                }

                // Fullscreen
                // debug!("{} {}", event.ctrl_key(), event.key().to_lowercase());
                if event.ctrl_key() && &event.key().to_lowercase() == "f" {
                    drawing_surfaces
                        .borrow()
                        .visible
                        .canvas
                        .request_fullscreen()
                        .unwrap();
                }
            },
        );
    }
}

// TODO: trait for frontends
fn draw_timeline(engine: Rc<RefCell<Engine>>, drawing_surfaces: Rc<RefCell<DrawingSurfaces>>) {
    // debug!("draw_timeline");
    set_canvas_sizes(&engine, &drawing_surfaces);
    clear_timeline(&drawing_surfaces);
    draw_backgrounds(&engine, &drawing_surfaces);
    draw_lines(&engine, &drawing_surfaces);
    draw_entities(&engine, &drawing_surfaces);
    draw_headings(&engine, &drawing_surfaces);
    // debug!("[exit] draw_timeline");
}

fn draw_headings(engine: &Rc<RefCell<Engine>>, drawing_surfaces: &Rc<RefCell<DrawingSurfaces>>) {
    // debug!("draw_headings");
    let headings_for_drawing = engine.borrow_mut().headings_for_drawing();
    let font_size = engine.borrow().effective_font_size_px();
    for mut heading in headings_for_drawing {
        // Draw visible
        draw_coloured_rect(&drawing_surfaces.borrow().visible.ctx, heading.text_box);

        // Draw invisible
        heading.text_box.fill_colour = Colour::from_rgb(0, 0, 0);
        draw_coloured_rect(&drawing_surfaces.borrow().invisible.ctx, heading.text_box);

        // Draw text
        draw_text(
            &drawing_surfaces.borrow().visible.ctx,
            font_size,
            heading.text,
        );
    }
}

fn draw_entities(engine: &Rc<RefCell<Engine>>, drawing_surfaces: &Rc<RefCell<DrawingSurfaces>>) {
    // debug!("draw_entities");
    let surfaces = drawing_surfaces.borrow();
    let visible_ctx = surfaces.visible.ctx.clone();
    let invisible_ctx = surfaces.invisible.ctx.clone();
    let entities_for_drawing = engine.borrow().entities_for_drawing();
    let font_size = engine.borrow().effective_font_size_px();
    // debug!(
    //     "Entities for drawing count = {}",
    //     entities_for_drawing.len()
    // );
    // debug!(
    //     "Entity count in engine = {}",
    //     engine.borrow().entity_count()
    // );
    for mut entity in entities_for_drawing {
        // Draw visible
        draw_coloured_rect(&visible_ctx, entity.text_box);
        draw_coloured_rect(&visible_ctx, entity.date_box);
        draw_text(&visible_ctx, font_size, entity.text);

        // Draw invisible
        let hidden_colour = Colour::from_any_string(entity.entity.name().as_str());
        entity.text_box.fill_colour = hidden_colour;
        entity.date_box.fill_colour = hidden_colour;

        draw_coloured_rect(&invisible_ctx, entity.text_box);
        draw_coloured_rect(&invisible_ctx, entity.date_box);
    }
}

fn draw_backgrounds(engine: &Rc<RefCell<Engine>>, drawing_surfaces: &Rc<RefCell<DrawingSurfaces>>) {
    // debug!("draw_backgrounds");
    let visible_ctx = &drawing_surfaces.borrow().visible.ctx;
    let visible_canvas_height = drawing_surfaces.borrow().visible.canvas.height();
    let backgrounds_for_drawing = engine.borrow().backgrounds_for_drawing();
    for background in backgrounds_for_drawing {
        // Draw visible
        let (r, g, b) = background.colour.as_rgb();
        visible_ctx.set_fill_style_str(&format!("rgba({r}, {g}, {b}, 1.0)"));
        visible_ctx.fill_rect(
            background.x,
            0.0,
            background.width,
            visible_canvas_height as f64,
        );
    }
}

fn draw_lines(engine: &Rc<RefCell<Engine>>, drawing_surfaces: &Rc<RefCell<DrawingSurfaces>>) {
    // debug!("draw_lines");
    let visible_ctx = &drawing_surfaces.borrow().visible.ctx;
    let visible_canvas_height = drawing_surfaces.borrow().visible.canvas.height();
    let lines_for_drawing = engine.borrow().lines_for_drawing();
    for line in lines_for_drawing {
        visible_ctx.begin_path();
        visible_ctx.move_to(line.x, 0.0);
        visible_ctx.line_to(line.x, visible_canvas_height as f64);
        let (r, g, b) = line.style.colour.as_rgb();
        visible_ctx.set_stroke_style_str(&format!("rgba({r}, {g}, {b}, 1.0)"));
        visible_ctx.set_line_width(line.style.thickness);
        visible_ctx.stroke();
    }
}

fn draw_coloured_rect(ctx: &CanvasRenderingContext2d, rect: FilledBox) {
    // debug!("draw_coloured_rect");
    // TODO: also the border colour and width
    let (r, g, b) = rect.fill_colour.as_rgb();
    ctx.set_fill_style_str(&format!("rgba({r}, {g}, {b}, 1.0)"));
    let x = rect.position_and_size.position.x;
    let y = rect.position_and_size.position.y;
    let width = rect.position_and_size.width;
    let height = rect.position_and_size.height;
    ctx.fill_rect(x, y, width, height);
}

fn draw_text(ctx: &CanvasRenderingContext2d, font_size: f64, text: TextOut) {
    // debug!("draw_text");
    let x = text.top_left.x;
    let mut y = text.top_left.y;
    let measurements = measure_text(font_size, &text.text);
    y += measurements.actual_bounding_box_ascent();
    let (r, g, b) = text.colour.as_rgb();
    ctx.set_fill_style_str(&format!("rgba({r}, {g}, {b}, 1.0)"));
    let _ = ctx.fill_text(&text.text, x, y);
}

fn clear_timeline(drawing_surfaces: &Rc<RefCell<DrawingSurfaces>>) {
    // debug!("clear_timeline");

    let height = drawing_surfaces.borrow().visible.canvas.height().into();
    let width = drawing_surfaces.borrow().visible.canvas.width().into();

    drawing_surfaces
        .borrow()
        .visible
        .ctx
        .clear_rect(0.0, 0.0, width, height);
    drawing_surfaces
        .borrow()
        .invisible
        .ctx
        .clear_rect(0.0, 0.0, width, height);
}

fn device_pixel_ratio() -> f64 {
    web_sys::window().unwrap().device_pixel_ratio()
}

fn colour_at_point(
    drawing_surfaces: &Rc<RefCell<DrawingSurfaces>>,
    x: f64,
    y: f64,
) -> Result<Colour, JsValue> {
    let image_data = drawing_surfaces
        .borrow()
        .invisible
        .ctx
        .get_image_data(x, y, 1.0, 1.0)?;
    let pixels = image_data.data();
    let r = pixels[0];
    let g = pixels[1];
    let b = pixels[2];
    let _a = pixels[3];
    let colour = Colour::from_rgb(r, g, b);
    debug!("colour = {colour:?}");
    Ok(colour)
}

fn set_canvas_sizes(engine: &Rc<RefCell<Engine>>, drawing_surfaces: &Rc<RefCell<DrawingSurfaces>>) {
    let dpr = device_pixel_ratio();

    // Gets cleared, so need to be temporarily saved and then re-set
    let context_font = drawing_surfaces.borrow().visible.ctx.font();

    let visible_canvas = &drawing_surfaces.borrow().visible.canvas;
    let invisible_canvas = &drawing_surfaces.borrow().invisible.canvas;
    let window = web_sys::window().unwrap();

    if window.document().unwrap().fullscreen() {
        // info!("Window is fullscreen");
        let window_inner_height = window.inner_height().unwrap().as_f64().unwrap();
        let window_inner_width = window.inner_width().unwrap().as_f64().unwrap();

        engine.borrow_mut().set_canvas_max(
            window_inner_width * device_pixel_ratio(),
            window_inner_height * device_pixel_ratio(),
        );

        for canvas in [visible_canvas, invisible_canvas] {
            canvas.set_width((window_inner_width * dpr) as u32);
            canvas.set_height((window_inner_height * dpr) as u32);
            canvas
                .style()
                .set_property("width", &format!("{window_inner_width}px"))
                .unwrap();
            canvas
                .style()
                .set_property("height", &format!("{window_inner_height}px"))
                .unwrap();
        }
    } else {
        //
        for canvas in [visible_canvas, invisible_canvas] {
            //
            let parent_width = canvas
                .parent_element()
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap()
                .client_width();
            let parent_height = canvas
                .parent_element()
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap()
                .client_height();

            engine.borrow_mut().set_canvas_max(
                parent_width as f64 * device_pixel_ratio(),
                parent_height as f64 * device_pixel_ratio(),
            );

            //
            canvas.set_width((parent_width as f64 * dpr) as u32);
            canvas.set_height((parent_height as f64 * dpr) as u32);
            canvas
                .style()
                .set_property("width", &format!("{parent_width}px"))
                .unwrap();
            canvas
                .style()
                .set_property("height", &format!("{parent_height}px"))
                .unwrap();
        }
    }

    // Re-set font
    drawing_surfaces
        .borrow()
        .visible
        .ctx
        .set_font(&context_font);
    drawing_surfaces
        .borrow()
        .invisible
        .ctx
        .set_font(&context_font);
}
