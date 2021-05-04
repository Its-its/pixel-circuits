use serde_json::Value as JsonValue;

use crate::Result;
use crate::objects::{ObjectData, Dimensions};
use super::{ObjectType, ObjectState, CellPos, Node, NodeValue, NodeValueTypes, NodeObjectSide, ObjectUpdateEvent, Creatable, ObjectId};
use super::Renderable;


pub struct ButtonState {
	state: ObjectState
}

impl Creatable for ButtonState {
	fn default_dim() -> Dimensions {
		Dimensions(1, 1)
	}

	fn new_with_opts(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable> {
		Box::new(
			Self {
				state: ObjectState::new_opts(id, ObjectType::Button, position, dimensions.unwrap_or_else(Self::default_dim), JsonValue::Null)
			}
		)
	}

	fn new_with_nodes(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable> {
		let mut item = Self::new_with_opts(id, position, dimensions);

		item.add_node(Node::new_output(NodeObjectSide::Right(0), NodeValueTypes::Gpio));
		item.add_node(Node::new_output(NodeObjectSide::Left(0), NodeValueTypes::Gpio));
		item.add_node(Node::new_output(NodeObjectSide::Top(0), NodeValueTypes::Gpio));
		item.add_node(Node::new_output(NodeObjectSide::Bottom(0), NodeValueTypes::Gpio));

		item
	}
}

impl Renderable for ButtonState {
	fn current_value(&self) -> NodeValue {
		NodeValue::Gpio(false)
	}

	fn name(&self) -> String {
		"Button".into()
	}

	fn display_name(&self) -> String {
		String::from("Btn")
	}

	fn get_object_state(&self) -> &ObjectState {
		&self.state
	}

	fn get_object_state_mut(&mut self) -> &mut ObjectState {
		&mut self.state
	}
}


impl Tickable for ButtonState {
	fn tick(&mut self, event: ObjectUpdateEvent) -> Result<Vec<ObjectData>> {
		if let ObjectUpdateEvent::MouseClickObject(_) = event {
			let outputs = self.state.nodes.iter()
				.filter(|n| n.direction.is_output());
			Ok(outputs.map(|n| n.send_output(self.get_id(), NodeValue::Gpio(true))).collect())
		} else {
			Ok(Vec::new())
		}
	}

	fn is_clickable(&self) -> bool {
		true
	}
}


register_tickable!(ButtonState);
