use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::rc::Rc;
use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use js_sys::Date;
use web_sys::{
	HtmlDivElement,
	MouseEvent as HtmlMouseEvent,
	WheelEvent as HtmlWheelEvent,
	KeyboardEvent as HtmlKeyboardEvent
};

use crate::{
	window,
	Result,
	canvas::Canvas,
	error::EditingError,
	statics::{
		get_editor_state,
		get_editor_state_mut,
		is_editor_running
	},
	ui::MainUi,
	objects::ObjectUpdateEvent
};

pub mod node;
pub mod state;
pub mod event;
pub mod util;
pub mod ui;

pub use node::*;
pub use event::{MouseEvent, WheelEvent, KeyboardEvent, MouseButton};
pub use util::{Cursor, ViewOptions};
pub use state::{EditorState, CanvasState};

macro_rules! create_event {
	($name:expr, $editor_event:ident, $html_event:ident, $listener:expr, $inner:expr) => {{
		let inner = $inner;
		let closure = Closure::wrap(Box::new(move |event: $html_event| {
			let mut mutable = inner.write().unwrap();
			mutable.update(EditorEvent::$editor_event(event.into()), &inner).expect("MOUSE MOVE EVENT UPDATE");
		}) as Box<dyn FnMut(_)>);
		$listener.add_event_listener_with_callback($name, closure.as_ref().unchecked_ref())?;
		closure.forget();
	}};
}

#[derive(Clone)]
pub struct Editor(Rc<RwLock<InnerEditor>>);

impl Editor {
	pub fn new(width: usize, height: usize) -> Self {
		Self(Rc::new(RwLock::new(InnerEditor::new(width, height))))
	}

	pub fn read(&self) -> Result<RwLockReadGuard<'_, InnerEditor>> {
		self.0.read().map_err(|_| EditingError::PoisonError.into())
	}

	pub fn write(&self) -> Result<RwLockWriteGuard<'_, InnerEditor>> {
		self.0.write().map_err(|_| EditingError::PoisonError.into())
	}


	pub fn init(&mut self, parent: &HtmlDivElement) -> Result<()> {
		let mut writable = self.write()?;
		writable.init(parent)?;

		{ // Prevent Context Menu
			let closure = Closure::wrap(Box::new(move |event: HtmlMouseEvent| {
				event.prevent_default();
			}) as Box<dyn FnMut(_)>);
			writable.canvas.element.add_event_listener_with_callback("contextmenu", closure.as_ref().unchecked_ref())?;
			closure.forget();
		}

		// Mouse Move Event
		create_event!("mousemove", MouseMove, HtmlMouseEvent, writable.canvas.element, self.clone());

		// Mouse Down Event
		create_event!("mousedown", MouseDown, HtmlMouseEvent, writable.canvas.element, self.clone());

		// Mouse Up Event
		create_event!("mouseup", MouseUp, HtmlMouseEvent, writable.canvas.element, self.clone());

		// Mouse Scroll Event
		create_event!("wheel", Scroll, HtmlWheelEvent, writable.canvas.element, self.clone());

		// Key Up Event
		create_event!("keyup", KeyUp, HtmlKeyboardEvent, crate::window(), self.clone());

		// Key Down Event
		create_event!("keydown", KeyDown, HtmlKeyboardEvent, crate::window(), self.clone());

		Ok(())
	}

	pub fn render(&self) {
		let inner = self.clone();


		let f = Rc::new(RefCell::new(None));
		let g = f.clone();

		*g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
			let mut mutable = inner.write().unwrap();

			if let Err(e) = mutable.update(EditorEvent::GlobalTick, &inner) {
				log!("Update Error: {:?}", e);
			}

			if let Err(e) = mutable.render() {
				log!("Render Error: {:?}", e);
			}

			request_animation_frame(f.borrow().as_ref().unwrap());
		}) as Box<dyn FnMut()>));

		request_animation_frame(g.borrow().as_ref().unwrap());
	}
}


pub struct InnerEditor {
	pub main_ui: Option<MainUi>,
	pub view_opts: ViewOptions,

	pub global_update_every: f64,
	pub global_last_update: f64,

	pub canvas: Canvas
}

impl InnerEditor {
	pub fn new(width: usize, height: usize) -> Self {
		Self {
			canvas: Canvas::new(),
			view_opts: ViewOptions::new(width, height),
			main_ui: None,

			global_update_every: 1000.0,
			global_last_update: Date::now()
		}
	}


	pub fn init(&mut self, parent: &HtmlDivElement) -> Result<()> {
		parent.append_with_node_1(self.canvas.as_ref())?;

		self.resize(self.view_opts.canvas_width(), self.view_opts.canvas_height());

		get_editor_state_mut().init(self)?;

		Ok(())
	}

	pub fn resize(&mut self, width: usize, height: usize) {
		self.canvas.set_width(width);
		self.canvas.set_height(height);

		self.view_opts.set_width(width);
		self.view_opts.set_height(height);
	}

	pub fn render(&self) -> Result<()> {
		let ctx = &self.canvas;

		ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
		ctx.clear_rect(0.0, 0.0, self.view_opts.width_f64(), self.view_opts.height_f64());
		ctx.set_font("12px Verdana");

		get_editor_state().render(self)?;

		Ok(())
	}

	pub fn update(&mut self, mut event: EditorEvent, editor_clone: &Editor) -> Result<()> {
		{ // Core Events
			match &mut event {
				EditorEvent::GlobalTick => {
					if is_editor_running() {
						let curr_time = Date::now();

						if self.global_last_update + self.global_update_every < curr_time {
							log!("Tick");

							let state = get_editor_state_mut();

							if let Some(canvas_state) = state.get_canvas_state_mut() {
								for o in &mut canvas_state.pixels.objects {
									if let Some(ticker) = o.as_tickable_mut() {
										crate::statics::add_to_ticking(ticker.tick(ObjectUpdateEvent::GlobalTick)?);
									}
								}
							}

							crate::statics::tick()?;

							self.global_last_update = curr_time;
						}
					}
				}

				EditorEvent::MouseDown(mouse) => {
					self.view_opts.on_mouse_down(mouse);
				}

				EditorEvent::MouseUp(mouse) => {
					self.view_opts.on_mouse_up(mouse);

					// Stop Canvas Dragging
					self.view_opts.stop_dragging_canvas();
				}

				EditorEvent::MouseMove(mouse) => {
					if let Some(button) = self.view_opts.button_pressed.as_ref().copied() {
						mouse.button = button;
					}

					self.view_opts.set_cursor_pos(mouse.fixed_x, mouse.fixed_y);

					// Canvas Dragging
					if let Some((start_x, start_y)) = self.view_opts.get_dragging_canvas_start_pos() {
						let new_x = (start_x - mouse.fixed_x).max(0.0);
						let new_y = (start_y - mouse.fixed_y).max(0.0);

						self.view_opts.set_drag_pos(new_x, new_y);
					}
				}

				EditorEvent::Scroll(mouse) => {
					self.view_opts.update_zoom(mouse.direction);
				}

				EditorEvent::KeyDown(keyboard) | EditorEvent::KeyUp(keyboard) => {
					if keyboard.key == "Backspace" {
						keyboard.event.prevent_default();
					}
				}

				_ => ()
			}
		}

		get_editor_state_mut().update(self, &event)?;

		// Artificial MouseClick since "click" event only listens to left clicks.
		if let EditorEvent::MouseUp(mouse) = event {
			self.update(EditorEvent::MouseClick(mouse), editor_clone)?;
		}

		Ok(())
	}
}


pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}




pub fn toggled_edit_mode() -> Result<()> {
	let editor = get_editor_state_mut();
	let canvas = editor.get_canvas_state_mut().unwrap();

	canvas.pixels.reset_all()?;

	Ok(())
}

#[derive(Debug, Clone)]
pub enum EditorEvent {
	MouseMove(MouseEvent),
	MouseDown(MouseEvent),
	MouseUp(MouseEvent),

	MouseClick(MouseEvent),

	// MouseDblClick(MouseEvent),

	// MouseDrag(f64, f64),
	// MouseDragEnd(f64, f64),
	// MouseDragStart(f64, f64),

	KeyDown(KeyboardEvent),
	KeyUp(KeyboardEvent),

	Scroll(WheelEvent),

	GlobalTick
}


pub struct Step {
	//
}