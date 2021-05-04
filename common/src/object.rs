use serde::{Deserialize, Serialize};

pub const MARGIN_SIZE: f64 = 3.0;


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NodeValue  {
	Gpio(bool)
}

impl NodeValue {
	pub fn is_active(self) -> bool {
		match self {
			Self::Gpio(v) => v
		}
	}

	pub fn unset(&mut self) {
		match self {
			Self::Gpio(v) => *v = false,
		}
	}
}



#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize)]
pub enum NodeValueTypes {
	Gpio,
	Current,
	GpioOrCurrent
}

impl PartialEq for NodeValueTypes {
	fn eq(&self, other: &Self) -> bool {
		matches!(
			(self, other),
			(Self::GpioOrCurrent, Self::Current) |
			(Self::GpioOrCurrent, Self::Gpio) |
			(Self::Current, Self::GpioOrCurrent) |
			(Self::Gpio, Self::GpioOrCurrent) |
			(Self::Current, Self::Current) |
			(Self::Gpio, Self::Gpio)
		)
	}
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum ObjectType {
	Button,
	Clock,
	Led,
	Switch,

	AndGate,
	NotGate,
	OrGate,
	XorGate,

	TestObj
}

impl ObjectType {
	pub fn list() -> Vec<ObjectType> {
		vec![
			ObjectType::Button,
			// ObjectType::Clock,
			ObjectType::Led,
			ObjectType::Switch,

			// ObjectType::AndGate,
			// ObjectType::NotGate,
			// ObjectType::OrGate,
			// ObjectType::XorGate,

			ObjectType::TestObj
		]
	}
}