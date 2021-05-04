use web_sys::{MouseEvent as HtmlMouseEvent, WheelEvent as HtmlWheelEvent, KeyboardEvent as HtmlKeyboardEvent};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
	Left,
	Right,
	Other(i16),
}

impl MouseButton {
	pub fn left(self) -> bool {
		matches!(self, Self::Left)
	}

	pub fn right(self) -> bool {
		matches!(self, Self::Right)
	}
}

impl From<i16> for MouseButton {
	fn from(button: i16) -> Self {
		match button {
			0 => MouseButton::Left,
			2 => MouseButton::Right,
			_ => MouseButton::Other(button)
		}
	}
}

impl Into<i16> for MouseButton {
	fn into(self) -> i16 {
		match self {
			MouseButton::Left => 0,
			MouseButton::Right => 2,
			MouseButton::Other(i) => i
		}
	}
}


#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
	pub fixed_x: f64,
	pub fixed_y: f64,
	pub button: MouseButton
}

impl From<HtmlMouseEvent> for MouseEvent {
	fn from(event: HtmlMouseEvent) -> Self {
		Self {
			fixed_x: event.offset_x() as f64,
			fixed_y: event.offset_y() as f64,
			button: event.button().into()
		}
	}
}


#[derive(Debug, Clone, Copy)]
pub struct WheelEvent {
	pub fixed_x: f64,
	pub fixed_y: f64,
	pub direction: i32
}

impl From<HtmlWheelEvent> for WheelEvent {
	fn from(event: HtmlWheelEvent) -> Self {
		Self {
			fixed_x: event.offset_x() as f64,
			fixed_y: event.offset_y() as f64,
			direction: if event.delta_y() > 0.0 { 1 } else { -1 }
		}
	}
}


#[derive(Debug, Clone)]
pub struct KeyboardEvent {
	pub event: HtmlKeyboardEvent,

	/// Will be affected if shift key is down for ex: press = "r" or shift + press = "R"
	pub key: String,

	pub is_held_down: bool,

	pub is_shift_down: bool,
	pub is_alt_down: bool,
	pub is_ctrl_down: bool,
	pub is_meta_down: bool,
}

impl From<HtmlKeyboardEvent> for KeyboardEvent {
	fn from(event: HtmlKeyboardEvent) -> Self {
		Self {
			key: event.key(),
			is_held_down: event.repeat(),
			is_shift_down: event.shift_key(),
			is_alt_down: event.alt_key(),
			is_ctrl_down: event.ctrl_key(),
			is_meta_down: event.meta_key(),
			event
		}
	}
}