use circuit_sim_common::{CellPos, NodeDirection, NodeObjectSide, NodeValueTypes, config::NodeJson, object::NodeValue};

use crate::{ids::ObjectId, objects::ObjectData};



#[derive(Debug)]
pub struct Node {
	pub cell_pos: CellPos,
	pub side: NodeObjectSide,
	pub direction: NodeDirection,

	pub accepts: NodeValueTypes,

	pub is_disabled: bool
}

impl Node {
	pub fn new(side: NodeObjectSide, direction: NodeDirection, accepts: NodeValueTypes) -> Self {
		Self {
			side,
			direction,
			accepts,

			is_disabled: false,
			cell_pos: (0, 0)
		}
	}

	pub fn new_input(side: NodeObjectSide, accepts: NodeValueTypes) -> Self {
		Self::new(side, NodeDirection::Input, accepts)
	}

	pub fn new_output(side: NodeObjectSide, accepts: NodeValueTypes) -> Self {
		Self::new(side, NodeDirection::Output, accepts)
	}

	pub fn send_output(&self, object_id: ObjectId, value: NodeValue) -> ObjectData {
		ObjectData::FromNode(object_id, self.cell_pos, self.side, value)
	}
}

impl Into<NodeJson> for &Node {
	fn into(self) -> NodeJson {
		NodeJson {
			pos: self.cell_pos,
			side: self.side,
			direction: self.direction,
			accepts: self.accepts,
			is_disabled: self.is_disabled,
			label: None
		}
	}
}

impl From<NodeJson> for Node {
	fn from(value: NodeJson) -> Self {
		Node {
			cell_pos: value.pos,
			side: value.side,
			direction: value.direction,
			accepts: value.accepts,
			is_disabled: value.is_disabled
		}
	}
}