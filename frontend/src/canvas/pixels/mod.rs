use std::collections::HashMap;

use circuit_sim_common::{CellPos, Side, object::NodeValue};

use crate::{InnerEditor, Result, editor::Node, ids::ObjectId, objects::Renderable};


mod map;
mod util;

pub use map::*;
pub use util::*;


pub static PIXEL_WHITE: PixelColor = PixelColor(255, 255, 255);

pub static PIXEL_ODD: PixelColor = PixelColor(100, 100, 100);
pub static PIXEL_OBJECT: PixelColor = PixelColor(82, 82, 82);
pub static PIXEL_NODE: PixelColor = PixelColor(214, 214, 214);

// https://coolors.co/c4c4be-cfaf9d-f0b78e-f4d892-a1d0c5-94c1e5-b89ce1-e693c1-f38f95
// https://coolors.co/919186-ad7758-e47b2f-eab52d-5cad9a-3f8fd1-7d4cc8-d23e8f-e92e3a


pub static PIXEL_TOOLS: [PixelColor; 1] = [
	PixelColor(237, 174, 192)
];


pub struct PixelBackground {
	pub palette: Vec<(PixelColor, PixelColor)>,
	pub cells: HashMap<(usize, usize), Pixel>,
	pub objects: Vec<Box<dyn Renderable>>
}

impl PixelBackground {
	pub fn new() -> Self {
		Self {
			palette: vec![
				(PixelColor(196, 196, 190), PixelColor(145, 145, 134)),
				(PixelColor(207, 175, 157), PixelColor(173, 119, 88)),
				(PixelColor(240, 183, 142), PixelColor(228, 123, 47)),
				(PixelColor(244, 216, 146), PixelColor(234, 181, 45)),
				(PixelColor(161, 208, 197), PixelColor(92, 173, 154)),
				(PixelColor(148, 193, 229), PixelColor(63, 143, 209)),
				(PixelColor(184, 156, 225), PixelColor(125, 76, 200)),
				(PixelColor(230, 147, 193), PixelColor(210, 62, 143)),
				(PixelColor(243, 143, 149), PixelColor(233, 46, 58)),
			],
			cells: HashMap::new(),
			objects: Vec::new()
		}
	}

	pub fn get_surrounding_cells((x, y): CellPos) -> Vec<(CellPos, Side)> {
		let mut surroundings = vec![((x, y + 1), Side::Bottom), ((x + 1, y), Side::Right)];

		if let Some(pos) = x.checked_sub(1).map(|x| (x, y)) {
			surroundings.push((pos, Side::Left));
		}

		if let Some(pos) = y.checked_sub(1).map(|y| (x, y)) {
			surroundings.push((pos, Side::Top));
		}

		surroundings
	}

	pub fn render(&self, editor: &InnerEditor) -> Result<()> {
		let ctx = &editor.canvas;

		let pixel_size = editor.view_opts.pixel_size as f64;

		for (&(x, y), type_of) in &self.cells {
			type_of.render(x, y, pixel_size, &self.palette, ctx)?;
		}


		for obj in &self.objects {
			obj.render(false, &editor.view_opts, ctx)?;
		}

		Ok(())
	}

	//

	pub fn get_pixel_type_from_cell_mut(&mut self, cell: &CellPos) -> Option<&mut Pixel> {
		self.cells.get_mut(cell)
	}


	// Wires

	pub fn get_wire_color(&self, cell: CellPos) -> Option<PixelColor> {
		self.cells.get(&cell)
			.and_then(|pixel| {
				if let PixelType::Wire { index, .. } = &pixel.type_of {
					Some(self.palette[*index].0)
				} else {
					None
				}
			})
	}

	pub fn get_wire_value(&self, cell: CellPos) -> Option<&NodeValue> {
		self.cells.get(&cell)
			.and_then(|pixel| {
				if let PixelType::Wire { value, .. } = &pixel.type_of {
					Some(value)
				} else {
					None
				}
			})
	}

	pub fn get_wire_value_mut(&mut self, cell: CellPos) -> Option<&mut NodeValue> {
		self.cells.get_mut(&cell)
			.and_then(|pixel| {
				if let PixelType::Wire { value, .. } = &mut pixel.type_of {
					Some(value)
				} else {
					None
				}
			})
	}

	pub fn set_wire(&mut self, cell: CellPos, pixel: PixelColor) {
		if let Some(pixel) = self.cells.get(&cell) {
			if !pixel.type_of.is_wire() {
				return;
			}
		}

		if let Some(index) = self.palette.iter().position(|p| p.0 == pixel) {
			self.insert_cell(cell, PixelType::Wire { index, value: NodeValue::Gpio(false) });
		}
	}

	pub fn is_wire_same_palette_index(&self, cell: CellPos, palette_index: usize) -> Option<bool> {
		self.cells.get(&cell)
			.and_then(|pixel| {
				if let PixelType::Wire { index, .. } = &pixel.type_of {
					Some(palette_index == *index)
				} else {
					None
				}
			})
	}


	// Pixel  Cell
	pub fn find_connectable_sides(&self, current_pixel_type: &PixelType, (x, y): CellPos) -> (bool, bool, bool, bool) {
		let top = y.checked_sub(1).map(|y| self.can_connect_to_cell(current_pixel_type, (x, y))).unwrap_or_default();
		let bottom = self.can_connect_to_cell(current_pixel_type, (x, y + 1));
		let left = x.checked_sub(1).map(|x| self.can_connect_to_cell(current_pixel_type, (x, y))).unwrap_or_default();
		let right = self.can_connect_to_cell(current_pixel_type, (x + 1, y));

		(top, bottom, left, right)
	}

	pub fn can_connect_to_cell(&self, current_pixel_type: &PixelType, cell_pos: CellPos) -> bool {
		if let PixelType::Wire { index, .. } = current_pixel_type {
			self.is_cell_node(cell_pos) || self.is_wire_same_palette_index(cell_pos, *index).unwrap_or_default()
		} else if current_pixel_type.is_node() {
			self.cells.contains_key(&cell_pos)
		} else {
			false
		}
	}

	pub fn delete_cell(&mut self, cell_pos: CellPos) {
		self.cells.remove(&cell_pos);

		self.update_surrounding_cells(cell_pos, false);
	}

	pub fn insert_cell(&mut self, (x, y): CellPos, pixel_type: PixelType) {
		if self.cells.contains_key(&(x, y)) {
			self.delete_cell((x, y));
		}

		let display = match pixel_type {
		    PixelType::Wire { .. } => {
				let sides = self.find_connectable_sides(&pixel_type, (x, y));
				PixelDisplay::new_from_sides(sides.0, sides.1, sides.2, sides.3)
			}

		    PixelType::Node { side, .. } => {
				let sides = self.find_connectable_sides(&pixel_type, (x, y));

				match side {
					Side::Top => PixelDisplay::new_from_sides(sides.0, true, sides.2, sides.3),
					Side::Bottom => PixelDisplay::new_from_sides(true, sides.1, sides.2, sides.3),
					Side::Left => PixelDisplay::new_from_sides(sides.0, sides.1, sides.2, true),
					Side::Right => PixelDisplay::new_from_sides(sides.0, sides.1, true, sides.3),
				}
			}

			_ => {
				self.cells.insert((x, y), Pixel {
					type_of: pixel_type,
					display: PixelDisplay::new_from_sides(false, false, false, false)
				});
				return;
			}
		};

		self.cells.insert((x, y), Pixel { type_of: pixel_type, display });
		self.update_surrounding_cells((x, y), true);
	}

	pub fn is_cell_node(&self, cell_pos: CellPos) -> bool {
		self.cells.get(&cell_pos).map(|p| p.type_of.is_node()).unwrap_or_default()
	}

	pub fn is_cell_wire(&self, cell_pos: CellPos) -> bool {
		self.cells.get(&cell_pos).map(|p| p.type_of.is_wire()).unwrap_or_default()
	}

	pub fn update_surrounding_cells(&mut self, cell_pos: CellPos, value: bool) {
		// Current cells' palette index, if it exists.
		let palette_index = self.cells.get(&cell_pos)
			.and_then(|pixel| {
				if let PixelType::Wire { index, .. } = &pixel.type_of {
					Some(*index)
				} else {
					None
				}
			});

		for (cell_pos, side) in Self::get_surrounding_cells(cell_pos) {
			if let Some(pixel) = self.cells.get_mut(&cell_pos) {
				match &pixel.type_of {
					PixelType::Node { .. } => {
						pixel.update_side(side.opposite(), value);
					}

					PixelType::Wire { .. } if palette_index.is_none() => {
						pixel.update_side(side.opposite(), value);
					}

					PixelType::Wire { index, .. } if Some(*index) == palette_index => {
						pixel.update_side(side.opposite(), value);
					}

					_=> ()
				}
			}
		}
	}


	// Objects

	pub fn re_render_objects(&mut self) {
		for object_id in self.objects.iter().map(|v| v.get_id()).collect::<Vec<_>>() {
			self.insert_object_cells_by_id(object_id);
		}
	}


	pub fn delete_object(&mut self, object_id: ObjectId) {
		self.delete_object_cells(object_id);

		if let Some(index) = self.objects.iter().position(|o| o.get_id() == object_id) {
			self.objects.remove(index);
		}
	}

	pub fn delete_object_cells(&mut self, object_id: ObjectId) {
		let object = self.get_object_by_id(object_id).unwrap();

		object.pixel_map()
			.pixel_positions(object.get_cell_pos())
			.into_iter()
			.for_each(|c| self.delete_cell(c));
	}

	pub fn insert_object_cells_by_id(&mut self, object_id: ObjectId) {
		// TODO: Temp since update_surrounding_cells mess up Node Cells.
		self.delete_object_cells(object_id);

		let object = self.get_object_by_id(object_id).unwrap();

		object.pixel_map()
			.pixel_positions_with_types(object.get_cell_pos())
			.into_iter()
			.for_each(|(cell, type_of)| self.insert_cell(cell, type_of));
	}


	pub fn add_object(&mut self, object: Box<dyn Renderable>) {
		self.objects.push(object);
	}


	pub fn get_node_in_cell(&self, cell: CellPos) -> Option<&Node> {
		self.get_object_in_cell(cell)
		.and_then(|object| object.get_node_from_cell_pos(&cell))
	}


	pub fn get_object_in_cell(&self, cell: CellPos) -> Option<&dyn Renderable> {
		self.objects.iter().find(|o| o.is_cell_inside(cell)).map(|v| &**v)
	}

	pub fn get_object_in_cell_mut(&mut self, cell: CellPos) -> Option<&mut Box<dyn Renderable>> {
		self.objects.iter_mut().find(|o| o.is_cell_inside(cell))
	}


	pub fn get_object_by_id(&self, id: ObjectId) -> Option<&dyn Renderable> {
		self.objects.iter().find(|o| o.get_id() == id).map(|v| &**v)
	}

	pub fn get_object_by_id_mut(&mut self, id: ObjectId) -> Option<&mut Box<dyn Renderable>> {
		self.objects.iter_mut().find(|o| o.get_id() == id)
	}


	pub fn is_valid_object_pos(&self, id: ObjectId) -> bool {
		let moving = match self.objects.iter().find(|v| v.get_id() == id) {
			Some(v) => v,
			None => return true
		};

		for obj in &self.objects {
			if obj.get_id() != id && moving.is_overlapping(&**obj) {
				return false;
			}
		}

		true
	}

	//

	pub fn reset_all(&mut self) -> Result<()> {
		for pixel in self.cells.values_mut() {
			if let PixelType::Wire { value, .. } = &mut pixel.type_of {
				value.unset();
			}
		}

		for object in &mut self.objects {
			object.reset_values()?;
		}

		for object_id in self.objects.iter().map(|v| v.get_id()).collect::<Vec<_>>() {
			self.insert_object_cells_by_id(object_id);
		}

		Ok(())
	}
}