use super::{ObjectType, ObjectState, CellPos, Node, NodeValue, NodeValueTypes, NodeObjectSide, ObjectUpdateEvent, Creatable, ObjectId};
use super::Renderable;

use crate::{
	canvas::{PixelMap, PixelColor, PixelType},
	objects::{ObjectData, Dimensions}
};
use crate::Result;

pub struct SwitchState {
	state: ObjectState,

	active: bool,
}

impl Creatable for SwitchState {
	fn default_dim() -> Dimensions {
		Dimensions(1, 1)
	}

	fn new_with_opts(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable> {
		Box::new(
			Self {
				state: ObjectState::new_opts(id, ObjectType::Switch, position, dimensions.unwrap_or_else(Self::default_dim), false),
				active: false
			}
		)
	}

	fn new_with_nodes(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable> {
		let mut item = Self::new_with_opts(id, position, dimensions);

		item.add_node(Node::new_output(NodeObjectSide::Left(0), NodeValueTypes::Gpio));
		item.add_node(Node::new_output(NodeObjectSide::Right(0), NodeValueTypes::Gpio));

		item
	}
}

impl SwitchState {
	pub fn is_toggled(&self) -> bool {
		self.active
	}

	fn toggle(&mut self) {
		self.active = !self.active;
	}
}

impl Renderable for SwitchState {
	fn pixel_map(&self) -> PixelMap {
		let pixel = if self.is_toggled() {
			PixelType::Custom(PixelColor(66, 251, 85))
		} else {
			PixelType::Custom(PixelColor(237, 66, 18))
		};

		PixelMap::generate_with_object_map(
			self.state.dimensions,
			vec![pixel],
			&self.state.nodes
		)
	}

	fn current_value(&self) -> NodeValue {
		NodeValue::Gpio(self.is_toggled())
	}

	fn name(&self) -> String {
		String::from("Switch")
	}

	fn display_name(&self) -> String {
		String::from(if self.is_toggled() { "1" } else { "0" })
	}

	fn get_object_state(&self) -> &ObjectState {
		&self.state
	}

	fn get_object_state_mut(&mut self) -> &mut ObjectState {
		&mut self.state
	}

	fn reset_values(&mut self) -> Result<()> {
		self.active = self.state.get_default_saved();

		Ok(())
	}
}


impl Tickable for SwitchState {
	fn tick(&mut self, event: ObjectUpdateEvent) -> Result<Vec<ObjectData>> {
		if let ObjectUpdateEvent::MouseClickObject(_) = &event {
			self.toggle();
		}

		if self.is_toggled() && event.is_global_tick() {
			Ok(
				self.state.nodes.iter()
					.filter_map(|n| {
						if n.direction.is_output() {
							Some(n.send_output(self.get_id(), NodeValue::Gpio(self.is_toggled())))
						} else {
							None
						}
					})
					.collect()
			)
		} else {
			Ok(Vec::new())
		}
	}

	fn is_clickable(&self) -> bool {
		true
	}
}


register_tickable!(SwitchState);