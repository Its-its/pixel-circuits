use circuit_sim_common::{CellPos, Dimensions, NodeObjectSide};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::Result;
use crate::editor::{Node, ViewOptions};

use super::{PIXEL_NODE, PIXEL_OBJECT, PixelColor, PixelType};





// Used for rendering the object.
#[derive(Debug)]
pub struct PixelMap {
	map: Vec<Option<PixelType>>,
	width: usize
}

impl PixelMap {
	/// map will NOT include Nodes. Only object.
	pub fn generate_with_object_map(dimensions: Dimensions, object_map: Vec<PixelType>, nodes: &[Node]) -> Self {
		let width = dimensions.width();

		let mut map = Vec::new();
		map.resize(width + 2, None);

		object_map.into_iter()
			.enumerate()
			.for_each(|(i, p)| {
				let x = i % width;

				if x == 0 {
					map.push(None);
				}

				map.push(Some(p));

				if x == width - 1 {
					map.push(None);
				}
			});

		(0..width + 2).for_each(|_| map.push(None));

		let mut this = Self {
			map,
			width: width + 2
		};

		this.render_nodes(nodes);

		this
	}

	pub fn new_empty(dimensions: Dimensions) -> Self {
		let width = dimensions.width() + 2;
		let height = dimensions.height() + 2;

		let mut map = Vec::new();
		map.resize(width * height, None);

		Self {
			map,
			width
		}
	}

	pub fn new_with_nodes(dimensions: Dimensions, nodes: &[Node]) -> Self {
		Self::new_empty(dimensions).generate(nodes)
	}


	pub fn generate(mut self, nodes: &[Node]) -> Self {
		self.render_object();
		self.render_nodes(nodes);

		self
	}


	fn render_object(&mut self) {
		// Skip first row. Ignore last row.
		(self.width .. self.map.len() - self.width)
		.for_each(|i| {
			let x = i % self.width;

			if x != 0 && x != self.width - 1 {
				self.map[i] = Some(PixelType::ObjectColor);
			}
		});
	}

	fn render_nodes(&mut self, nodes: &[Node]) {
		nodes.iter().for_each(|n| {
			let i = match n.side {
				NodeObjectSide::Top(v) => 1 + v,
				NodeObjectSide::Bottom(v) => self.map.len() - v - 2,
				NodeObjectSide::Right(v) => self.width * 2 + self.width * v - 1,
				NodeObjectSide::Left(v) => self.map.len() - (self.width * 2) - (self.width * v),
			};

			self.map[i] = Some(PixelType::Node { direction: n.direction, side: n.side.side() });
		});
	}

	pub fn pixel_positions(&self, pos: CellPos) -> Vec<CellPos> {
		self.map.iter()
			.enumerate()
			.filter_map(|(i, t)| {
				if t.is_some() {
					let x_offset = i % self.width;
					let y_offset = i / self.width;

					Some((
						(pos.0 + x_offset).checked_sub(1)?,
						(pos.1 + y_offset).checked_sub(1)?
					))
				} else {
					None
				}
			})
			.collect()
	}

	pub fn pixel_positions_with_types(self, pos: CellPos) -> Vec<(CellPos, PixelType)> {
		let width = self.width;

		self.map.into_iter()
			.enumerate()
			.filter_map(|(i, t)| {
				if let Some(type_of) = t {
					let x_offset = i % width;
					let y_offset = i / width;

					Some((
						((pos.0 + x_offset).checked_sub(1)?, (pos.1 + y_offset).checked_sub(1)?),
						type_of
					))
				} else {
					None
				}
			})
			.collect()
	}

	pub fn render(&self, pos: CellPos, _is_selected: bool, view: &ViewOptions, palette: &[(PixelColor, PixelColor)], ctx: &CanvasRenderingContext2d) -> Result<()> {
		let pixel_size = view.pixel_size as f64;

		let rendering_pixels = self.map.iter()
			.enumerate()
			.filter_map(|(i, t)| t.as_ref().map(|t| (i, t)));

		for (i, type_of) in rendering_pixels {
			let x_offset = i % self.width;
			let y_offset = i / self.width;

			if let (Some(pos_x), Some(pos_y)) = ((pos.0 + x_offset).checked_sub(1), (pos.1 + y_offset).checked_sub(1)) {
				let pixel = match type_of {
					PixelType::Custom(p) => *p,
					PixelType::Node { .. } => PIXEL_NODE,
					PixelType::ObjectColor => PIXEL_OBJECT,
					PixelType::Wire { index, .. } => palette[*index].0
				};

				ctx.set_fill_style(&JsValue::from_str(&pixel.get_string_color()));

				ctx.fill_rect(
					pos_x as f64 * pixel_size,
					pos_y as f64 * pixel_size,
					pixel_size,
					pixel_size
				);
			}
		}

		Ok(())
	}
}
