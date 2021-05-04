// https://learn.sparkfun.com/tutorials/light-emitting-diodes-leds/all

// Current
// Continuous: 20mA
// Burst: 30mA
// Suggested Range: 16mA-18mA
// Max Damage: > 105mA

// Voltage
// Forward Voltage: 1.8-2.2v
// Wavelength: 620-625nm
// Luminous Intensity: 150-200mcd

use serde_json::Value as JsonValue;

use crate::{Result, canvas::{
		PixelMap,
		PixelType,
		PixelColor
	}, objects::ObjectUpdateEvent};
use super::{ObjectType, ObjectState, CellPos, Node, NodeValue, NodeValueTypes, NodeObjectSide, Creatable, ObjectId};
use super::Renderable;

use crate::objects::{ObjectData, Dimensions};


// Allows Current or GPIO.
// Checks output connection to ensure outputting proper Value.

pub struct LedState {
	state: ObjectState,

	last_value: NodeValue
}

impl Creatable for LedState {
	fn default_dim() -> Dimensions {
		Dimensions(1, 1)
	}

	fn new_with_opts(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable> {
		Box::new(
			Self {
				state: ObjectState::new_opts(id, ObjectType::Led, position, dimensions.unwrap_or_else(Self::default_dim), JsonValue::Null),
				last_value: NodeValue::Gpio(false)
			}
		)
	}

	fn new_with_nodes(id: ObjectId, position: CellPos, dimensions: Option<Dimensions>) -> Box<dyn Renderable> {
		let mut item = Self::new_with_opts(id, position, dimensions);

		item.add_node(Node::new_input(NodeObjectSide::Left(0), NodeValueTypes::GpioOrCurrent));
		item.add_node(Node::new_output(NodeObjectSide::Right(0), NodeValueTypes::GpioOrCurrent));

		item
	}
}

impl LedState {
	pub fn is_lit(&self) -> bool {
		self.last_value.is_active()
	}
}

impl Renderable for LedState {
	fn pixel_map(&self) -> PixelMap {
		let pixel = PixelType::Custom(
			if self.is_lit() {
				PixelColor(190, 224, 31)
			} else {
				PixelColor(80, 100, 15)
			}
		);

		PixelMap::generate_with_object_map(
			self.state.dimensions,
			vec![pixel],
			&self.state.nodes
		)
	}

	fn current_value(&self) -> NodeValue {
		self.last_value
	}

	fn on_receive(&mut self, _: NodeObjectSide, value: NodeValue) -> Result<Option<Vec<ObjectData>>> {
		self.last_value = value;

		let nodes: Vec<_> = self.state.nodes.iter()
			.filter_map(|n| {
				if n.direction.is_output() {
					Some(n.send_output(self.get_id(), self.current_value()))
				} else {
					None
				}
			})
			.collect();

		if nodes.is_empty() {
			Ok(None)
		} else {
			Ok(Some(nodes))
		}
	}

	fn name(&self) -> String {
		"LED".into()
	}

	fn display_name(&self) -> String {
		String::from("LED")
	}

	fn get_object_state(&self) -> &ObjectState {
		&self.state
	}

	fn get_object_state_mut(&mut self) -> &mut ObjectState {
		&mut self.state
	}

	fn reset_values(&mut self) -> Result<()> {
		self.last_value = NodeValue::Gpio(false);

		Ok(())
	}
}


impl Tickable for LedState {
    fn tick(&mut self, event: ObjectUpdateEvent) -> Result<Vec<ObjectData>> {
		if event.is_global_tick() {
			self.last_value.unset();
		}

		Ok(Vec::new())
    }
}

register_tickable!(LedState);