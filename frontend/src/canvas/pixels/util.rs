use circuit_sim_common::{NodeDirection, Side, object::NodeValue};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::Result;
use crate::canvas::{PIXEL_NODE, PIXEL_OBJECT};




#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelColor(pub u16, pub u16, pub u16);

impl PixelColor {
	pub fn from_integer(value: u32) -> Self {
		let r = (value >> 16) & 0xFF;
		let g = (value >> 8) & 0xFF;
		let b = value & 0xFF;

		Self(r as u16, g as u16, b as u16)
	}

	pub fn into_integer(self) -> u32 {
		let mut rgb = self.0 as u32;
		rgb = (rgb << 8) | self.1 as u32;
		rgb = (rgb << 8) | self.2 as u32;
		rgb
	}

	pub fn get_string_color(self) -> String {
		format!("rgb({}, {}, {})", self.0, self.1, self.2)
	}
}

impl From<(u16, u16, u16)> for PixelColor {
	fn from(value: (u16, u16, u16)) -> Self {
		Self(value.0, value.1, value.2)
	}
}

impl Into<(u16, u16, u16)> for PixelColor {
	fn into(self) -> (u16, u16, u16) {
		(self.0, self.1, self.2)
	}
}



#[derive(Debug, Clone, Copy)]
pub struct PixelDisplay(pub u8);

#[allow(clippy::fn_params_excessive_bools)]
impl PixelDisplay {
	pub fn new_from_sides(top: bool, bottom: bool, left: bool, right: bool) -> Self {
		let mut byte = 0;

		if top { byte |= 0b0001; }
		byte <<= 1;

		if bottom { byte |= 0b0001; }
		byte <<= 1;

		if left { byte |= 0b0001; }
		byte <<= 1;

		if right { byte |= 0b0001; }

		Self(byte)
	}

	pub fn update_side(&mut self, side: Side, value: bool) {
		let pos = match side {
			Side::Top => 3,
			Side::Bottom => 2,
			Side::Left => 1,
			Side::Right => 0
		};

		let mask = 1 << pos;
		let set_bit = if value { mask } else { 0b0000 };

		self.0 = (self.0 & !mask) | set_bit;
	}


	pub fn render(self, cell_x: usize, cell_y: usize, pixel_size: f64, pixel: PixelColor, ctx: &CanvasRenderingContext2d) {
		let inner_size = Self::get_size(pixel_size);

		let re_align = pixel_size / 2.0 - inner_size / 2.0;

		ctx.set_fill_style(&JsValue::from_str(&pixel.get_string_color()));

		let pos_x = cell_x as f64 * pixel_size;
		let pos_y = cell_y as f64 * pixel_size;

		// Center
		ctx.fill_rect(
			pos_x + re_align,
			pos_y + re_align,
			inner_size,
			inner_size
		);

		// log!("{}", inner_size);

		// Top
		if self.0 & (1 << 3) != 0 {
			ctx.fill_rect(
				pos_x + re_align,
				pos_y,
				inner_size,
				inner_size
			);
		}

		// Bottom
		if self.0 & (1 << 2) != 0 {
			ctx.fill_rect(
				pos_x + re_align,
				pos_y + re_align * 2.0,
				inner_size,
				inner_size
			);
		}

		// Left
		if self.0 & (1 << 1) != 0 {
			ctx.fill_rect(
				pos_x,
				pos_y + re_align,
				inner_size,
				inner_size
			);
		}

		// Right
		if self.0 & (1 << 0) != 0 {
			ctx.fill_rect(
				pos_x + re_align * 2.0,
				pos_y + re_align,
				inner_size,
				inner_size
			);
		}
	}

	fn get_size(pixel_size: f64) -> f64 {
		pixel_size / 2.8
	}
}



#[derive(Debug, Clone)]
pub enum PixelType {
	Wire {
		index: usize,
		value: NodeValue
	},

	Node {
		direction: NodeDirection,
		side: Side
	},

	Custom(PixelColor),

	ObjectColor,
}

impl PixelType {
	pub fn is_wire(&self) -> bool {
		matches!(self, Self::Wire { .. })
	}

	pub fn is_node(&self) -> bool {
		matches!(self, Self::Node { .. })
	}

	pub fn get_wire_palette_index(&self) -> Option<usize> {
		if let Self::Wire { index, .. } = self {
			Some(*index)
		} else {
			None
		}
	}
}


#[derive(Debug, Clone)]
pub struct Pixel {
	pub type_of: PixelType,
	pub display: PixelDisplay
}

impl Pixel {
	pub fn render(&self, cell_x: usize, cell_y: usize, pixel_size: f64, palette: &[(PixelColor, PixelColor)], ctx: &CanvasRenderingContext2d) -> Result<()> {
		if self.type_of.is_wire() || self.type_of.is_node() {
			let pixel = match &self.type_of {
				PixelType::Wire { index, value } => if value.is_active() {
					palette[*index].1
				} else {
					palette[*index].0
				},
				PixelType::Node { .. } => PIXEL_NODE,

				_ => unreachable!()
			};

			self.display.render(
				cell_x,
				cell_y,
				pixel_size,
				pixel,
				ctx
			);
		} else {
			let pixel = match &self.type_of {
				PixelType::Custom(p) => *p,
				PixelType::ObjectColor => PIXEL_OBJECT,

				_ => unreachable!()
			};

			ctx.set_fill_style(&JsValue::from_str(&pixel.get_string_color()));

			ctx.fill_rect(
				cell_x as f64 * pixel_size,
				cell_y as f64 * pixel_size,
				pixel_size,
				pixel_size
			);
		}

		Ok(())
	}

	pub fn update_side(&mut self, side: Side, value: bool) {
		if self.type_of.is_wire() || self.type_of.is_node() {
			self.display.update_side(side, value);
		}
	}
}
