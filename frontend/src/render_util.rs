use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;


pub fn fill_rect_centered(x: f64, y: f64, w: f64, h: f64, ctx: &CanvasRenderingContext2d) {
	ctx.fill_rect(x - (w/2.0), y - (h/2.0), w, h);
}

pub fn fill_triangle_centered(x: f64, y: f64, ctx: &CanvasRenderingContext2d) {
	let _ = ctx.translate(x, y);
	ctx.begin_path();
	ctx.move_to(-5.0, 0.0);
	ctx.line_to(5.0, 5.0);
	ctx.line_to(5.0, -5.0);
	ctx.fill();
	let _ = ctx.translate(-x, -y);
}


// Connections

pub static CONNECTION_GPIO: LineColor<'static> = LineColor::new(
	"#11f081", "#0dd572",
	"#0AA558", "#000"
);

pub static CONNECTION_CURRENT: LineColor<'static> = LineColor::new(
	"#1a8b97", "#0F5258",
	"#0d444a", "#000"
);

pub static CONNECTION_NONE: LineColor<'static> = LineColor::new(
	"#C0B213", "#000",
	"#C0B213", "#000"
);


// pub fn get_line_colors_by_type(type_of: &LineValue) -> &LineColor<'static> {
// 	match type_of {
// 		LineValue::Gpio(_) => &CONNECTION_GPIO,
// 		LineValue::Electricity(_, _) => &CONNECTION_CURRENT,
// 		LineValue::None => &CONNECTION_NONE
// 	}
// }

// // TODO: Advanced render fn. Ex: render(start_pos, type_of, ctx).to(next_pos).to(next_pos).finish()

// pub fn render_connection_on_cells(
// 	start: CellPos,
// 	end: CellPos,
// 	type_of: &LineValue,
// 	ctx: &CanvasRenderingContext2d
// ) {
// 	let start = (
// 		start.0 as f64 * CELL_SIZE + CELL_SIZE_HALF,
// 		start.1 as f64 * CELL_SIZE + CELL_SIZE_HALF
// 	);

// 	let end = (
// 		end.0 as f64 * CELL_SIZE + CELL_SIZE_HALF,
// 		end.1 as f64 * CELL_SIZE + CELL_SIZE_HALF
// 	);

// 	render_connection(start, end, type_of, ctx);
// }

// pub fn render_connection(
// 	(start_x, start_y): CanvasPos,
// 	(end_x, end_y): CanvasPos,
// 	type_of: &LineValue,
// 	ctx: &CanvasRenderingContext2d
// ) {
// 	let color = get_line_colors_by_type(type_of);

// 	// Outline

// 	color.stroke_outline(type_of.is_active() || type_of.is_powered(), ctx);

// 	ctx.set_line_width(5.0);

// 	ctx.begin_path();

// 	ctx.move_to(start_x, start_y);
// 	ctx.line_to(end_x, end_y);

// 	ctx.stroke();

// 	// Inner

// 	color.stroke_inline(type_of.is_active() || type_of.is_powered(), ctx);

// 	ctx.set_line_width(3.0);

// 	ctx.begin_path();

// 	ctx.move_to(start_x, start_y);
// 	ctx.line_to(end_x, end_y);

// 	ctx.stroke();
// }


pub struct Color<'a> {
	pub inline: &'a str,
	pub outline: &'a str
}

pub struct LineColor<'a> {
	pub on: Color<'a>,
	pub off: Color<'a>
}

impl<'a> LineColor<'a> {
	pub const fn new(on_inline: &'a str, on_outline: &'a str, off_inline: &'a str, off_outline: &'a str) -> Self {
		Self {
			on: Color { inline: on_inline, outline: on_outline },
			off: Color { inline: off_inline, outline: off_outline }
		}
	}

	pub fn stroke(&self, inline: bool, active: bool, ctx: &CanvasRenderingContext2d) {
		let color = if active {
			&self.on
		} else {
			&self.off
		};

		ctx.set_stroke_style(&JsValue::from_str(
			if inline {
				color.inline
			} else {
				color.outline
			}
		));
	}

	pub fn stroke_inline(&self, active: bool, ctx: &CanvasRenderingContext2d) {
		self.stroke(true, active, ctx)
	}

	pub fn stroke_outline(&self, active: bool, ctx: &CanvasRenderingContext2d) {
		self.stroke(false, active, ctx)
	}
}