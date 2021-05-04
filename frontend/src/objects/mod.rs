use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d};

use circuit_sim_common::{
	CanvasPos, NodeValueTypes, NodeDirection, Side,
	object::NodeValue
};

use crate::{
	CellPos, Result,
	canvas::{PixelMap, PixelType},
	editor::{CanvasState, Cursor, MouseEvent, Node, ViewOptions},
	ids::ObjectId
};


pub mod basic;
// pub mod gates;
pub mod testobj;

pub use basic::*;
// pub use gates::*;
pub use testobj::TestObjState;


use circuit_sim_common::object::ObjectType;
use circuit_sim_common::size::{NodeObjectSide, Dimensions};



pub fn create_new_object_from_type(type_of: ObjectType) -> Box<dyn Renderable> {
	let position = (5, 5);
	let id = ObjectId::gen_id();

	create_new_object(id, type_of, position, None)
}

pub fn create_new_object(id: ObjectId, type_of: ObjectType, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable> {
	match type_of {
		ObjectType::TestObj => TestObjState::new_with_nodes(id, position, dimensions),
		ObjectType::Switch => SwitchState::new_with_nodes(id, position, dimensions),
		ObjectType::Button => ButtonState::new_with_nodes(id, position, dimensions),
		ObjectType::Led => LedState::new_with_nodes(id, position, dimensions),

		v => unimplemented!("{:?}", v)
	}
}




pub struct ObjectState {
	pub id: ObjectId,
	pub type_of: ObjectType,

	cell_pos: CellPos,
	dimensions: Dimensions,

	pub nodes: Vec<Node>,

	default: JsonValue
}

impl ObjectState {
	pub fn new_opts<D: Into<JsonValue>>(id: ObjectId, type_of: ObjectType, cell_pos: CellPos, dimensions: Dimensions, default: D) -> Self {
		Self {
			id,
			type_of,
			cell_pos,
			dimensions,
			default: default.into(),

			nodes: Vec::new()
		}
	}

	pub fn get_canvas_pos(&self, cell_size: f64) -> CanvasPos {
		(self.cell_pos.0 as f64 * cell_size, self.cell_pos.1 as f64 * cell_size)
	}


	pub fn get_default_saved<D: DeserializeOwned>(&self) -> D {
		serde_json::from_value(self.default.clone()).unwrap()
	}


	pub fn get_dimensions(&self) -> Dimensions {
		self.dimensions
	}

	pub fn set_dimensions(&mut self, dimensions: Dimensions) {
		self.dimensions = dimensions;
		self.update_nodes();
	}

	pub fn get_cell_pos(&self) -> CellPos {
		self.cell_pos
	}

	pub fn set_cell_pos(&mut self, cell_pos: CellPos) {
		self.cell_pos = cell_pos;
		self.update_nodes();
	}


	fn update_nodes(&mut self) {
		let obj_dim = self.dimensions;

		for node in &mut self.nodes {
			if let Some(pos) = node.side.get_cell_pos(&obj_dim, self.cell_pos) {
				node.cell_pos = pos;
				node.is_disabled = false;
			} else {
				node.cell_pos = (0, 0);
				node.is_disabled = true;
			}
		}
	}
}



pub trait Renderable: ImplTickable {
	fn get_object_state(&self) -> &ObjectState;
	fn get_object_state_mut(&mut self) -> &mut ObjectState;

	fn get_dimensions(&self) -> Dimensions {
		self.get_object_state().get_dimensions()
	}

	fn set_dimensions(&mut self, dimensions: Dimensions) {
		self.get_object_state_mut().set_dimensions(dimensions);
	}

	fn get_cell_pos(&self) -> CellPos {
		self.get_object_state().get_cell_pos()
	}

	fn set_cell_pos(&mut self, cell_pos: CellPos) {
		self.get_object_state_mut().set_cell_pos(cell_pos);
	}

	fn get_node_from_cell_pos(&self, cell_pos: &CellPos) -> Option<&Node> {
		self.get_object_state().nodes.iter().find(|n| &n.cell_pos == cell_pos)
	}


	fn pixel_map(&self) -> PixelMap {
		PixelMap::new_with_nodes(self.get_object_state().dimensions, &self.get_object_state().nodes)
	}

	/// Returns the name of the Object.
	fn name(&self) -> String;
	fn display_name(&self) -> String;

	fn on_receive(&mut self, _node_side: NodeObjectSide, _value: NodeValue) -> Result<Option<Vec<ObjectData>>> {
		Ok(None)
	}

	fn on_output(&mut self, _node_side: NodeObjectSide) {}

	fn current_value(&self) -> NodeValue;

	fn reset_values(&mut self) -> Result<()> { Ok(()) }

	/// Rendering of the `Object`.
	fn render(
		&self,
		_is_selected: bool,
		view: &ViewOptions,
		ctx: &CanvasRenderingContext2d
	) -> Result<()> {
		let dim = self.get_object_state().dimensions;
		let cell_size = view.pixel_size as f64;

		let obj_pos = self.get_object_state().get_canvas_pos(cell_size);


		let width = dim.get_width_accountable(cell_size);
		let height = dim.get_height_accountable(cell_size);


		if view.pixel_size > 30 {
			ctx.set_text_align("center");
			ctx.set_text_baseline("middle");

			ctx.set_font("14px Verdana");

			ctx.set_fill_style(&JsValue::from_str("#a6a6a6"));

			ctx.fill_text(
				&self.display_name(),
				obj_pos.0 + (width / 2.0) + 2.0,
				obj_pos.1 + (height / 2.0) + 3.0
			)?;
		}

		Ok(())
	}


	/// Is the Object overlapping another one.
	fn is_overlapping(&self, other: &dyn Renderable) -> bool {
		let pix_pos_this = self.pixel_map().pixel_positions(self.get_object_state().cell_pos);
		let pix_pos_other = other.pixel_map().pixel_positions(other.get_object_state().cell_pos);

		for pos in pix_pos_this {
			if pix_pos_other.contains(&pos) {
				return true;
			}
		}

		false
	}

	/// Is Cell inside object.
	fn is_cell_inside(&self, value: CellPos) -> bool {
		let dim = self.get_object_state().dimensions;

		let (min_x, min_y) = self.get_object_state().cell_pos;
		let (max_x, max_y) = (min_x + dim.width() - 1, min_y + dim.height() - 1);

		let (p_x, p_y) = value;

		// In Object?
		(p_x >= min_x && p_y >= min_y && p_x <= max_x && p_y <= max_y)
		|| self.get_object_state().nodes
			.iter()
			.any(|n| n.side.get_cell_pos(&dim, (min_x, min_y)) == Some(value))
	}

	// Rename to is_point_inside or something of the sort.
	fn is_mouse_inside(&self, mouse: &Cursor) -> bool {
		mouse.is_cells_positive() && self.is_cell_inside(mouse.cell_usize_unchecked())
	}

	fn add_node(&mut self, node: Node) {
		self.get_object_state_mut().nodes.push(node);
	}

	fn get_id(&self) -> ObjectId {
		self.get_object_state().id
	}
}


// impl Deref for dyn Renderable {
// 	type Target = ObjectState;

// 	fn deref(&self) -> &Self::Target {
// 		self.get_object_state()
// 	}
// }

// impl DerefMut for dyn Renderable {
// 	fn deref_mut(&mut self) -> &mut Self::Target {
// 		self.get_object_state_mut()
// 	}
// }


pub trait Creatable {
	fn new_with_opts(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable>;
	fn new_with_nodes(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable>;

	fn default_dim() -> Dimensions {
		Dimensions(3, 3)
	}
}

// Ideas below from https://stackoverflow.com/a/30275713

/// If the Object can be resized, nodes remapped, ...
// pub trait Editable {
// 	fn max_dimensions(&self) -> Option<Dimensions> { None }
// 	fn min_dimensions(&self) -> Option<Dimensions> { None }

// 	fn editable_values(&self) -> Vec<NodeValueTypes>;
// }

// pub fn default_editable_nodes() -> Vec<NodeType> {
// 	vec![NodeType::Input, NodeType::Output]
// }

/// The ability to call the item when running.
pub trait Tickable {
	fn tick(&mut self, event: ObjectUpdateEvent) -> Result<Vec<ObjectData>>;

	// fn ui_objects(&self, _container: ObjectContainer) -> Option<Vec<UiObject>> { None }
	// fn ui_sidebar(&self, _object: ObjectContainer, _container: HtmlDivElement) -> Option<Result<ItemContainer>> { None }

	// If clicked item when running program.
	fn is_clickable(&self) -> bool { false }
}


// pub trait ImplEditable {
// 	fn as_editable_ref(&self) -> Option<&dyn Editable>;
// 	fn as_editable_mut(&mut self) -> Option<&mut dyn Editable>;
// }

pub trait ImplTickable {
	fn as_tickable_ref(&self) -> Option<&dyn Tickable>;
	fn as_tickable_mut(&mut self) -> Option<&mut dyn Tickable>;
}



pub enum ObjectUpdateEvent {
	MouseHoveringObject,
	MouseClickObject(MouseEvent),
	MouseDownObject(MouseEvent),
	MouseUpObject(MouseEvent),

	GlobalTick
}

impl ObjectUpdateEvent {
	pub fn is_global_tick(&self) -> bool {
		matches!(self, Self::GlobalTick)
	}
}


#[derive(Debug, Clone)]
pub enum ObjectData {
	/// Outputting data from Object.
	/// (Current Pos, Node Side, Receiving Value)
	FromNode(ObjectId, CellPos, NodeObjectSide, NodeValue),

	/// Returned when a line has branches. Each branch is returned in `Vec<ObjectData::Branch>`
	/// (Received Data From, Current Position).
	Wire(Side, CellPos)
}

impl ObjectData {
	pub fn continue_sending(self, canvas: &mut CanvasState) -> Option<Result<Vec<ObjectData>>> {
		let mut data = Vec::new();

		match self {
			ObjectData::FromNode(object_id, cell_pos_start, node_side, node_value) => {
				canvas.pixels.insert_object_cells_by_id(object_id);

				for (possible_cell_pos, moving) in &node_side.side().get_surrounding_cells_with_side_its_on(cell_pos_start) {
					if let Some(cell_pos_move) = possible_cell_pos {
						if let Some(pixel) = canvas.pixels.get_pixel_type_from_cell_mut(cell_pos_move) {

							match &mut pixel.type_of {
								PixelType::Node { direction: NodeDirection::Input, .. } => {
									let into_obj_id = canvas.pixels.get_object_in_cell_mut(*cell_pos_move)
										.map(|object| {
											if let Ok(Some(mut value)) = object.on_receive(node_side, node_value) {
												data.append(&mut value);
											}

											object.get_id()
										});

									if let Some(into_object_id) = into_obj_id {
										canvas.pixels.insert_object_cells_by_id(into_object_id);
									}
								}

								PixelType::Wire { value, .. } => {
									if &node_value != value {
										*value = node_value;
										data.push(ObjectData::Wire(moving.opposite(), *cell_pos_move));
									}
								}

								_ => ()
							}
						}
					}
				}
			}

			ObjectData::Wire(movement, start_cell_pos) => {
				let start_pixel = canvas.pixels.get_wire_color(start_cell_pos)?;
				let start_value = *canvas.pixels.get_wire_value(start_cell_pos)?;

				for (possible_cell_pos, moving) in &movement.opposite().get_surrounding_cells_with_side_its_on(start_cell_pos) {
					if let Some(cell_pos_move) = possible_cell_pos {
						let palette = canvas.pixels.palette.clone(); // TODO: Remove

						if let Some(pixel) = canvas.pixels.get_pixel_type_from_cell_mut(cell_pos_move) {

							match &mut pixel.type_of {
								PixelType::Node { direction: NodeDirection::Input, .. } => {
									let obj_id = canvas.pixels
										.get_object_in_cell_mut(*cell_pos_move)
										.and_then(|object| {
											let node_side = object.get_node_from_cell_pos(cell_pos_move)?.side;

											if let Ok(Some(mut value)) = object.on_receive(node_side, start_value) {
												data.append(&mut value);
											}

											Some(object.get_id())
										});

									if let Some(object_id) = obj_id {
										canvas.pixels.insert_object_cells_by_id(object_id);
									}
								}

								PixelType::Wire { index, value } => {
									let new_wire_pixel = palette[*index].0;

									if &start_value != value && start_pixel == new_wire_pixel {
										*value = start_value;
										data.push(ObjectData::Wire(moving.opposite(), *cell_pos_move));
									}
								}

								_ => ()
							}
						}
					}
				}

				// Unset Current Wire.
				canvas.pixels.get_wire_value_mut(start_cell_pos)?.unset();
			}
		}

		Some(Ok(data))
	}
}

impl PartialEq for ObjectData {
	fn eq(&self, other: &ObjectData) -> bool {
		match (self, other) {
			(ObjectData::FromNode(_, v1, _, _), ObjectData::FromNode(_, v2, _, _)) |
			(ObjectData::Wire(_, v1), ObjectData::Wire(_, v2)) => {
				v1 == v2
			}

			_ => false
		}
	}
}