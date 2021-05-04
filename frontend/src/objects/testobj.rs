use circuit_sim_common::object::NodeValue;
use serde_json::Value as JsonValue;

use crate::{Result, canvas::PixelMap, editor::Node};
use super::{CellPos, Creatable, NodeObjectSide, NodeValueTypes, ObjectId, ObjectState, ObjectType};
use super::Renderable;
use crate::objects::{ObjectData, Dimensions};



pub struct TestObjState {
	state: ObjectState,
	last_received: Option<NodeValue>
}

impl Creatable for TestObjState {
	fn new_with_opts(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable> {
		Box::new(Self {
			state: ObjectState::new_opts(id, ObjectType::TestObj, position, dimensions.unwrap_or_else(Self::default_dim), JsonValue::Null),
			last_received: None
		})
	}

	fn new_with_nodes(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable> {
		let mut item = Self::new_with_opts(id, position, dimensions);

		item.add_node(Node::new_input(NodeObjectSide::Left(1), NodeValueTypes::Gpio));
		item.add_node(Node::new_output(NodeObjectSide::Right(1), NodeValueTypes::Gpio));

		item
	}
}


impl Renderable for TestObjState {
	fn pixel_map(&self) -> PixelMap {
		PixelMap::new_with_nodes(self.state.dimensions, &self.state.nodes)
	}

	fn name(&self) -> String {
		"TestObj".into()
	}

	fn display_name(&self) -> String {
		String::from("Test")
	}

	fn get_object_state(&self) -> &ObjectState {
		&self.state
	}

	fn get_object_state_mut(&mut self) -> &mut ObjectState {
		&mut self.state
	}

	fn current_value(&self) -> NodeValue {
		self.last_received.unwrap_or(NodeValue::Gpio(false))
	}

	fn on_receive(&mut self, from_node: NodeObjectSide, value: NodeValue) -> Result<Option<Vec<ObjectData>>> {
		log!(" -> {:?}: {:?}", from_node, value);

		self.last_received = Some(value);

		let data = self.state.nodes.iter()
			.filter_map(|n| {
				if n.direction.is_output() {
					Some(n.send_output(self.get_id(), value))
				} else {
					None
				}
			})
			.collect();

		Ok(Some(data))
	}
}


register_tickable!(!TestObjState);