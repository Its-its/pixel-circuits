
use std::rc::Rc;
use std::sync::Mutex;
use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use web_sys::{
	HtmlDivElement, HtmlElement, HtmlCanvasElement,
	CanvasRenderingContext2d,
	MouseEvent as HtmlMouseEvent
};

use circuit_sim_common::Rectangle;

use crate::{editor::toggled_edit_mode, statics::{is_editor_running, set_editor_running}};
use crate::Result;
use crate::editor::{request_animation_frame, EditorEvent};
use crate::editor::ui::{Display, ObjectPosition, ButtonState};
use crate::editor::util::ViewOptions;

pub struct StartContainer {
	container: HtmlDivElement,

	canvas: Rc<Mutex<Canvas>>
}

impl StartContainer {
	pub fn new() -> Self {
		Self {
			container: crate::create_element::<HtmlDivElement>("div"),
			canvas: Rc::new(Mutex::new(Canvas::new())),
		}
	}

	pub fn render(&mut self, parent: &HtmlElement) -> Result<()> {
		parent.append_with_node_1(&self.container)?;

		{
			let mut canvas = self.canvas.lock()?;
			canvas.init(parent.client_width().max(240) as u32, self.canvas.clone())?;
			canvas.render(Some(&self.container))?;
		}

		let inner = self.canvas.clone();

		let f = Rc::new(RefCell::new(None));
		let g = f.clone();

		*g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
			let mut mutable = inner.lock().unwrap();

			if let Err(e) = mutable.update(EditorEvent::GlobalTick) {
				log!("Update Error: {:?}", e);
			}

			if let Err(e) = mutable.render(None) {
				log!("Render Error: {:?}", e);
			}

			request_animation_frame(f.borrow().as_ref().unwrap());
		}) as Box<dyn FnMut()>));

		request_animation_frame(g.borrow().as_ref().unwrap());


		Ok(())
	}
}

pub struct Canvas {
	element: HtmlCanvasElement,

	pub listeners: Vec<Closure<dyn FnMut(HtmlMouseEvent)>>,

	view: ViewOptions,

	buttons: Vec<ButtonState>
}

impl Canvas {
	pub fn new() -> Self {
		Self {
			view: ViewOptions::new(0, 65),
			element: crate::create_element::<HtmlCanvasElement>("canvas"),
			listeners: Vec::new(),
			buttons: Vec::new()
		}
	}

	pub fn init(&mut self, width: u32, cloned: Rc<Mutex<Canvas>>) -> Result<()> {
		self.view.set_width(width as usize);

		self.element.set_width(self.view.canvas_width() as u32);
		self.element.set_height(self.view.canvas_height() as u32);

		self.listeners.clear();

		{ // Down
			let inner = cloned.clone();
			let closure = Closure::wrap(Box::new(move |event: HtmlMouseEvent| {
				let mut lock = inner.lock().unwrap();
				lock.update(EditorEvent::MouseDown(event.into())).expect("UPDATE EVENT");
			}) as Box<dyn FnMut(_)>);
			self.element.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
			self.listeners.push(closure);
		}

		{ // Up
			let inner = cloned.clone();
			let closure = Closure::wrap(Box::new(move |event: HtmlMouseEvent| {
				let mut lock = inner.lock().unwrap();
				lock.update(EditorEvent::MouseUp(event.into())).expect("UPDATE EVENT");
			}) as Box<dyn FnMut(_)>);
			self.element.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
			self.listeners.push(closure);
		}

		{ // Move
			let inner = cloned;
			let closure = Closure::wrap(Box::new(move |event: HtmlMouseEvent| {
				let mut lock = inner.lock().unwrap();
				lock.update(EditorEvent::MouseMove(event.into())).expect("UPDATE EVENT");
			}) as Box<dyn FnMut(_)>);
			self.element.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
			self.listeners.push(closure);
		}

		{ // Play/Pause
			let button = ButtonState::new(
				Display::Render(Box::new(|ctx, size| {
					ctx.set_line_width(4.0);

					let center_y = size.height / 2.0;
					let margin_size = 10.0;

					if is_editor_running() {
						// Pause Icon
						let y_spacing = 5.0;

						ctx.set_stroke_style(&JsValue::from_str("#EEA"));

						ctx.begin_path();
						ctx.move_to(margin_size, center_y - y_spacing);
						ctx.line_to(size.width - margin_size, center_y - y_spacing);
						ctx.stroke();

						ctx.begin_path();
						ctx.move_to(margin_size, center_y + y_spacing);
						ctx.line_to(size.width - margin_size, center_y + y_spacing);
						ctx.stroke();
					} else {
						// Play Icon
						ctx.set_stroke_style(&JsValue::from_str("#AEA"));

						ctx.begin_path();
						ctx.move_to(size.width - margin_size, center_y);
						ctx.line_to(margin_size, size.height - margin_size);
						ctx.line_to(margin_size, margin_size);
						ctx.line_to(size.width - margin_size, center_y);
						ctx.stroke();
					}

					Ok(())
				})),
				ObjectPosition::Fixed(Rectangle::new(10.0, 10.0, self.view.height_f64() - 20.0, self.view.height_f64() - 20.0)),
				Box::new(move || {
					set_editor_running(!is_editor_running());
					toggled_edit_mode().unwrap();
				})
			);

			self.buttons.push(button);
		}

		Ok(())
	}

	pub fn render(&self, parent: Option<&HtmlElement>) -> Result<()> {
		if let Some(parent) = parent {
			parent.append_with_node_1(&self.element)?;
		}

		let cursor = self.view.cursor();

		let ctx: CanvasRenderingContext2d = self.element
			.get_context("2d")?
			.unwrap()
			.dyn_into::<CanvasRenderingContext2d>()
			.unwrap();

		ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
		ctx.clear_rect(0.0, 0.0, self.view.width_f64(), self.view.height_f64());

		for button in &self.buttons {
			button.render(&cursor, &ctx)?;
		}

		Ok(())
	}

	pub fn update(&mut self, event: EditorEvent) -> Result<()> {
		match &event {
			EditorEvent::MouseMove(mouse) => {
				self.view.set_cursor_pos(mouse.fixed_x, mouse.fixed_y);
			}

			EditorEvent::MouseDown(event) => {
				self.view.on_mouse_down(event);
			}

			EditorEvent::MouseUp(event) => {
				self.view.on_mouse_up(event);
			}

			_ => ()
		}

		let cursor = self.view.cursor();

		for button in &mut self.buttons {
			if button.update(&cursor, &event)?.is_some() {
				break;
			}
		}

		Ok(())
	}
}