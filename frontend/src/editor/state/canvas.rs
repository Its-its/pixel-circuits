use std::any::Any;

use circuit_sim_common::config::StateInfo;
use js_sys::Date;
use wasm_bindgen::prelude::*;



use crate::{
	Result,
	canvas::{
		PIXEL_TOOLS,
		PixelDisplay,
		PixelColor,
		PixelBackground
	},
	editor::{
		InnerEditor,
		EditorEvent
	},
	ids::ObjectId,
	objects::ObjectUpdateEvent,
	statics::{
		self,
		is_editor_running
	}
};

use super::EditorState;



#[derive(Debug)]
pub enum CanvasEvent {
	SelectedObject(ObjectId),
	MovingObject(ObjectId),

	Painting(PixelColor),
	Tooling(CanvasTool)
}


#[derive(Debug, Clone, Copy)]
pub enum CanvasTool {
	Eraser
}




pub struct CanvasState {
	pub info: StateInfo,

	pub pixels: PixelBackground,
	pub event: Option<CanvasEvent>
}

impl CanvasState {
	pub fn new() -> Self {
		Self {
			info: StateInfo::from_user_id(1, Date::now() as i64),
			pixels: PixelBackground::new(),
			event: None
		}
	}
}

impl EditorState for CanvasState {
	fn init(&mut self, editor: &mut InnerEditor) -> Result<()> {
		if let Some(ui) = editor.main_ui.as_ref() {
			ui.write()?.sidebar.init_item_containers()?;
			ui.write()?.sidebar.render_all();
		}

		Ok(())
	}

	fn render(&self, editor: &InnerEditor) -> Result<()> {
		let ctx = &editor.canvas;

		ctx.set_font("14px Verdana");

		let cursor = editor.view_opts.cursor();

		ctx.translate(
			-editor.view_opts.get_canvas_drag_x(),
			-editor.view_opts.get_canvas_drag_y()
		)?;

		let cells_in_view = editor.view_opts.cells_in_view();

		{ // Background
			ctx.set_fill_style(&JsValue::from_str("#2C2C2C"));

			let pixel_size = editor.view_opts.pixel_size as f64;

			for pos_x in 0..cells_in_view.2 {
				for pos_y in 0..cells_in_view.3 {
					let is_odd = (cells_in_view.0 + cells_in_view.1 + pos_x + pos_y + 1) % 2 == 1;

					if is_odd {
						ctx.fill_rect(
							(cells_in_view.0 as f64 + pos_x as f64) * pixel_size,
							(cells_in_view.1 as f64 + pos_y as f64) * pixel_size,
							pixel_size,
							pixel_size
						);
					}
				}
			}
		}

		self.pixels.render(editor)?;

		if let Some(event) = self.event.as_ref() {
			match event {
				CanvasEvent::Painting(p) => {
					let pixel_size = editor.view_opts.pixel_size as f64;

					let display = PixelDisplay(0);

					let (cell_x, cell_y) = cursor.cell_usize_checked();
					display.render(cell_x, cell_y, pixel_size, *p, ctx);
				}

				CanvasEvent::Tooling(tool) => match tool {
					CanvasTool::Eraser => {
						let pixel_size = editor.view_opts.pixel_size as f64;

						ctx.set_fill_style(&JsValue::from_str(&PIXEL_TOOLS[0].get_string_color()));

						ctx.fill_rect(
							cursor.cell_x() as f64 * pixel_size,
							cursor.cell_y() as f64 * pixel_size,
							pixel_size,
							pixel_size
						);
					}
				}

				// Selected Object
				CanvasEvent::SelectedObject(id) => { // TODO: Show deleted Nodes.
					if let Some(obj) = self.pixels.get_object_by_id(*id) {
						let cell_pos = obj.get_cell_pos();

						obj.pixel_map().render(cell_pos, true, &editor.view_opts, &self.pixels.palette, ctx)?;
					}
				}

				CanvasEvent::MovingObject(id) => {
					if let Some(obj) = self.pixels.get_object_by_id(*id) {
						let cell_pos = obj.get_cell_pos();

						obj.pixel_map().render(cell_pos, true, &editor.view_opts, &self.pixels.palette, ctx)?;
					}
				}
			}
		}


		// 2D
		if editor.view_opts.debug {
			ctx.reset_transform()?;

			ctx.set_text_align("left");
			ctx.set_text_baseline("top");

			ctx.set_fill_style(&JsValue::from_str("#EEE"));

			// Pixel Size / Cursor Pos / Cursor Grid Pos
			ctx.fill_text(&format!(
				"Pixel Size: {}, Grid ({}, {}), Mouse ({}, {})",
				editor.view_opts.pixel_size,
				cursor.cell_x(),
				cursor.cell_y(),
				cursor.mouse_x(),
				cursor.mouse_y()
			), 1.0, 1.0)?;

			// Cells in view
			ctx.fill_text(&format!(
				"In View: {:?}",
				cells_in_view
			), 1.0, 16.0)?;

			// Cells in view
			ctx.fill_text(&format!(
				"Event: {:?}",
				self.event
			), 1.0, 33.0)?;
		}

		Ok(())
	}


	fn update(&mut self, editor: &mut InnerEditor, editor_event: &EditorEvent) -> Result<()> {
		if is_editor_running() {
			update_running(self, editor, editor_event)
		} else {
			update_editing(self, editor, editor_event)
		}
	}


	fn as_any_ref(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}


	fn set_state_info(&mut self, value: StateInfo) {
		self.info = value;
	}

	fn get_state_info(&self) -> StateInfo {
		self.info.clone()
	}
}

fn update_running(this: &mut CanvasState, editor: &mut InnerEditor, editor_event: &EditorEvent) -> Result<()> {
	let cursor = editor.view_opts.cursor();

	let cell = cursor.cell_usize_checked();

	let mut update_obj_pixels = None;

	if let Some(object) = this.pixels.get_object_in_cell_mut(cell) {
		if let Some(tick) = object.as_tickable_mut() {
			let resp = match editor_event {
				EditorEvent::MouseClick(e) if e.button.left() => {
					tick.tick(ObjectUpdateEvent::MouseClickObject(*e))
				}

				EditorEvent::MouseDown(e) if e.button.left() => {
					tick.tick(ObjectUpdateEvent::MouseDownObject(*e))
				}

				EditorEvent::MouseUp(e) if e.button.left() => {
					tick.tick(ObjectUpdateEvent::MouseUpObject(*e))
				}

				EditorEvent::MouseMove(e) if e.button.left() => {
					tick.tick(ObjectUpdateEvent::MouseHoveringObject)
				}

				_ => return Ok(())
			}?;

			if !resp.is_empty() {
				statics::add_to_ticking(resp);
			}

			update_obj_pixels = Some(object.get_id());
		}
	} else {
		//
	}

	if let Some(object_id) = update_obj_pixels {
		this.pixels.insert_object_cells_by_id(object_id);
	}

	Ok(())
}


fn update_editing(this: &mut CanvasState, editor: &mut InnerEditor, editor_event: &EditorEvent) -> Result<()> {
	let cursor = editor.view_opts.cursor();

	if let Some(event) = this.event.as_ref() {
		match event {
			// Placing Wires
			CanvasEvent::Painting(p) => {
				let cell = cursor.cell_usize_checked();

				match editor_event {
					// Unset
					EditorEvent::MouseClick(e) if e.button.right() => {
						this.event.take();
						return Ok(());
					}

					// Place Wiring
					EditorEvent::MouseClick(e) if e.button.left() => {
						this.pixels.set_wire(cell, *p);
						return Ok(());
					}

					EditorEvent::MouseMove(e) if e.button.left() && editor.view_opts.is_mouse_dragging() => {
						this.pixels.set_wire(cell, *p);
						return Ok(());
					}

					_ => ()
				}
			}

			// Toolings
			CanvasEvent::Tooling(tool) => match tool {
				// Eraser
				CanvasTool::Eraser => {
					let cell_pos = cursor.cell_usize_checked();

					match editor_event {
						// Unset
						EditorEvent::MouseClick(e) if e.button.right() => {
							this.event.take();
							return Ok(());
						}

						EditorEvent::MouseClick(e) if e.button.left() => {
							if this.pixels.is_cell_wire(cell_pos) {
								this.pixels.delete_cell(cell_pos);
							}
							return Ok(());
						}

						EditorEvent::MouseMove(e) if e.button.left() && editor.view_opts.is_mouse_dragging() => {
							if this.pixels.is_cell_wire(cell_pos) {
								this.pixels.delete_cell(cell_pos);
							}
							return Ok(());
						}

						_ => ()
					}
				}
			}

			// Selected Object
			&CanvasEvent::SelectedObject(object_id) => {
				match editor_event {
					// Unset Object
					EditorEvent::MouseClick(e) if e.button.left() => {
						let obj = this.pixels.get_object_by_id(object_id).unwrap();

						if obj.get_cell_pos() != cursor.cell_usize_checked() {
							this.pixels.insert_object_cells_by_id(object_id);
							this.event.take();
							return Ok(());
						}

						//
					}

					// Start Moving Object
					EditorEvent::MouseMove(_) if editor.view_opts.is_last_button_press_left() => {
						if let Some(cell) = editor.view_opts.cursor_start_drag_cell {
							let obj = this.pixels.get_object_by_id(object_id).unwrap();

							if obj.get_cell_pos() == cell && cursor.cell_usize_checked() != cell {
								this.event = Some(CanvasEvent::MovingObject(object_id));
							}
						}
					}

					//

					_ => ()
				}
			}

			// Moving Object
			&CanvasEvent::MovingObject(object_id) => {
				let cell = cursor.cell_usize_checked();

				match editor_event {
					// Move object and check if can be moved there.
					EditorEvent::MouseMove(_) => {
						let obj = this.pixels.get_object_by_id_mut(object_id).unwrap();

						let size = obj.get_dimensions();

						let new_cell_pos = (
							cell.0.saturating_sub(size.width() / 2),
							cell.1.saturating_sub(size.height() / 2)
						);

						let last_cell_pos = obj.get_cell_pos();

						if new_cell_pos != last_cell_pos {
							obj.set_cell_pos(new_cell_pos);
						}

						if new_cell_pos != last_cell_pos && !this.pixels.is_valid_object_pos(object_id) {
							this.pixels.get_object_by_id_mut(object_id).unwrap().set_cell_pos(last_cell_pos);
						}

						return Ok(());
					}

					// Remove
					EditorEvent::MouseClick(e) if e.button.right() => {
						this.pixels.delete_object(object_id);
						this.event.take();
						return Ok(());
					}

					// Placed Object. Remove Wires in Object Cells. Unset Event
					EditorEvent::MouseClick(e) if e.button.left() => {
						this.pixels.insert_object_cells_by_id(object_id);
						this.event.take();
						return Ok(());
					}

					_ => ()
				}
			}
		}
	} else {
		match editor_event {
			EditorEvent::MouseClick(e) if e.button.left() => {
				let cell = cursor.cell_usize_checked();

				if let Some(id) = this.pixels.get_object_in_cell(cell).map(|o| o.get_id()) {
					this.pixels.delete_object_cells(id);
					this.event = Some(CanvasEvent::SelectedObject(id));
					return Ok(());
				}
			}

			_ => ()
		}
	}


	// Start Canvas Dragging
	if editor.view_opts.is_last_button_press_right() && !editor.view_opts.is_dragging_canvas() && editor.view_opts.is_mouse_dragging() {
		editor.view_opts.start_dragging_canvas();
	}

	Ok(())
}