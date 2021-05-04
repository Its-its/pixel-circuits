use circuit_sim_common::{CellPos, Rectangle};

use web_sys::CanvasRenderingContext2d;

use super::{MouseEvent, MouseButton};


// TODO: Make Static
#[derive(Default, Clone)]
pub struct ViewOptions {
	pub debug: bool,

	canvas_width: usize,
	canvas_height: usize,

	pub button_pressed: Option<MouseButton>,
	// cursor drag amount | Used to determine if actually dragging an object.
	pub cursor_start_drag: Option<(f64, f64)>,
	pub cursor_start_drag_cell: Option<CellPos>,

	cursor_pos_x: f64,
	cursor_pos_y: f64,

	canvas_drag_x: f64,
	canvas_drag_y: f64,

	dragging_canvas: Option<(f64, f64)>,

	pub pixel_size: usize,
}

impl ViewOptions {
	pub fn new(canvas_width: usize, canvas_height: usize) -> Self {
		Self {
			debug: true,
			pixel_size: 32,
			canvas_width,
			canvas_height,
			.. ViewOptions::default()
		}
	}

	pub fn reset(&mut self) {
		self.button_pressed = None;
		self.dragging_canvas = None;
		self.cursor_start_drag = None;
		self.canvas_drag_x = 0.0;
		self.canvas_drag_y = 0.0;
	}

	// Events
	pub fn on_mouse_up(&mut self, _event: &MouseEvent) {
		self.button_pressed = None;
		self.cursor_start_drag = None;
		self.cursor_start_drag_cell = None;
	}

	pub fn on_mouse_down(&mut self, event: &MouseEvent) {
		self.button_pressed = Some(event.button);
		self.cursor_start_drag = Some((self.get_mouse_x(), self.get_mouse_y()));
		self.cursor_start_drag_cell = Some(self.cursor().cell_usize_checked());
	}

	pub fn update_zoom(&mut self, mut value: i32) {
		if value != 0 {
			if self.pixel_size >= 20 {
				value *= 2;
			}

			if value > 0 {
				self.pixel_size += value as usize;
			} else {
				self.pixel_size = self.pixel_size.saturating_sub(value.abs() as usize).max(10);
			}
		}
	}

	pub fn is_last_button_press_right(&self) -> bool {
		self.button_pressed.map(MouseButton::right).unwrap_or_default()
	}

	pub fn is_last_button_press_left(&self) -> bool {
		self.button_pressed.map(MouseButton::left).unwrap_or_default()
	}

	/// Ensures we're actually going to drag.
	///
	/// Checks when we originally did `mouse down` and checks that we've moved from a distance of 4.
	pub fn is_mouse_dragging(&self) -> bool {
		if let Some((start_x, start_y)) = self.cursor_start_drag.as_ref() {
			(start_x - self.get_mouse_x()).abs() >= 4.0 ||
			(start_y - self.get_mouse_y()).abs() >= 4.0
		} else {
			false
		}
	}


	// Width / Height
	pub fn set_width(&mut self, canvas_width: usize) {
		self.canvas_width = canvas_width;
	}

	pub fn set_height(&mut self, canvas_height: usize) {
		self.canvas_height = canvas_height;
	}

	// Canvas canvas_width/canvas_height, not viewing (yet)
	pub fn canvas_width(&self) -> usize {
		self.canvas_width
	}

	pub fn canvas_height(&self) -> usize {
		self.canvas_height
	}

	pub fn width_f64(&self) -> f64 {
		self.canvas_width as f64
	}

	pub fn height_f64(&self) -> f64 {
		self.canvas_height as f64
	}


	// Mouse

	pub fn cursor(&self) -> Cursor {
		Cursor {
			cursor_pos_x: self.cursor_pos_x,
			cursor_pos_y: self.cursor_pos_y,

			canvas_drag_x: self.canvas_drag_x,
			canvas_drag_y: self.canvas_drag_y,

			pixel_size: self.pixel_size
		}
	}


	pub fn get_mouse_fixed_x(&self) -> f64 {
		self.cursor_pos_x
	}

	pub fn get_mouse_fixed_y(&self) -> f64 {
		self.cursor_pos_y
	}

	//
	pub fn get_mouse_x(&self) -> f64 {
		self.get_mouse_fixed_x() + self.get_canvas_drag_x()
	}

	pub fn get_mouse_y(&self) -> f64 {
		self.get_mouse_fixed_y() + self.get_canvas_drag_y()
	}

	pub fn set_cursor_pos(&mut self, cursor_x: f64, cursor_y: f64) {
		self.cursor_pos_x = cursor_x;
		self.cursor_pos_y = cursor_y;
	}

	// Dragging

	pub fn start_dragging_canvas(&mut self) {
		self.dragging_canvas = Some((self.get_mouse_fixed_x() + self.get_canvas_drag_x(), self.get_mouse_fixed_y() + self.get_canvas_drag_y()));
	}

	pub fn stop_dragging_canvas(&mut self) {
		self.dragging_canvas = None;
	}

	pub fn is_dragging_canvas(&self) -> bool {
		self.dragging_canvas.is_some()
	}

	pub fn get_dragging_canvas_start_pos(&self) -> Option<&(f64, f64)> {
		self.dragging_canvas.as_ref()
	}

	pub fn get_canvas_drag_x(&self) -> f64 {
		self.canvas_drag_x
	}

	pub fn get_canvas_drag_y(&self) -> f64 {
		self.canvas_drag_y
	}

	pub fn set_drag_pos(&mut self, canvas_drag_x: f64, canvas_drag_y: f64) {
		self.canvas_drag_x = canvas_drag_x;
		self.canvas_drag_y = canvas_drag_y;
	}

	pub fn set_drag_x(&mut self, canvas_drag_x: f64) {
		self.canvas_drag_x = canvas_drag_x;
	}

	pub fn set_drag_y(&mut self, canvas_drag_y: f64) {
		self.canvas_drag_y = canvas_drag_y;
	}

	// Other

	pub fn cells_in_view(&self) -> (usize, usize, usize, usize) {
		let drag_x = self.get_canvas_drag_x();
		let drag_y = self.get_canvas_drag_y();

		let pixel_size = self.pixel_size as f64;

		(
			(drag_x / pixel_size).floor() as usize,
			(drag_y / pixel_size).floor() as usize,
			(self.canvas_width as f64 / pixel_size).ceil() as usize + 1,
			(self.canvas_height as f64 / pixel_size).ceil() as usize + 1
		)
	}
}

#[derive(Default, Clone, Copy)]
pub struct Cursor {
	cursor_pos_x: f64,
	cursor_pos_y: f64,

	canvas_drag_x: f64,
	canvas_drag_y: f64,

	pixel_size: usize
}

impl Cursor {
	// Mouse

	pub fn is_cells_positive(&self) -> bool {
		self.cell_x().is_positive() &&
		self.cell_y().is_positive()
	}

	pub fn mouse_fixed_x(&self) -> f64 {
		self.cursor_pos_x
	}

	pub fn mouse_fixed_y(&self) -> f64 {
		self.cursor_pos_y
	}

	pub fn mouse_x(&self) -> f64 {
		self.mouse_fixed_x() + self.drag_x()
	}

	pub fn mouse_y(&self) -> f64 {
		self.mouse_fixed_y() + self.drag_y()
	}

	pub fn cell_x(&self) -> i32 {
		(self.mouse_x() / self.pixel_size as f64).floor() as i32
	}

	pub fn cell_y(&self) -> i32 {
		(self.mouse_y() / self.pixel_size as f64).floor() as i32
	}

	pub fn cell(&self) -> (i32, i32) {
		(self.cell_x(), self.cell_y())
	}

	pub fn cell_usize_unchecked(&self) -> CellPos {
		(self.cell_x() as usize, self.cell_y() as usize)
	}

	pub fn cell_usize_checked(&self) -> CellPos {
		(self.cell_x().max(0) as usize, self.cell_y().max(0) as usize)
	}

	// Dragging

	pub fn drag_x(&self) -> f64 {
		self.canvas_drag_x
	}

	pub fn drag_y(&self) -> f64 {
		self.canvas_drag_y
	}


	pub fn unset_drag(&mut self) {
		self.canvas_drag_x = 0.0;
		self.canvas_drag_y = 0.0;
	}

	pub fn is_inside(&self, x: f64, y: f64, width: f64, height: f64) -> bool {
		let (mx, my) = (self.mouse_x(), self.mouse_y());
		mx >= x && my >= y && mx <= x + width && my <= y + height
	}
}


pub trait RetangleOpts {
	fn is_cursor_inside(&self, cursor: &Cursor) -> bool;
	fn render_stroke(&self, ctx: &CanvasRenderingContext2d);
	fn render_fill(&self, ctx: &CanvasRenderingContext2d);
}


impl RetangleOpts for Rectangle<f64> {
	fn is_cursor_inside(&self, cursor: &Cursor) -> bool {
		let mouse_x = cursor.mouse_x();
		let mouse_y = cursor.mouse_y();

		self.x <= mouse_x &&
		self.y <= mouse_y &&
		self.x + self.width >= mouse_x &&
		self.y + self.height >= mouse_y
	}

	fn render_stroke(&self, ctx: &CanvasRenderingContext2d) {
		ctx.stroke_rect(self.x, self.y, self.width, self.height);
	}

	fn render_fill(&self, ctx: &CanvasRenderingContext2d) {
		ctx.fill_rect(self.x, self.y, self.width, self.height);
	}
}