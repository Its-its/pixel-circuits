use std::ops::{Deref, DerefMut};

use web_sys::CanvasRenderingContext2d;

// Displayable in both the canvas and html.

use circuit_sim_common::Rectangle;

mod container;
mod button;
mod input;

pub use input::InputType;
pub use button::ButtonState;

use crate::Result;

pub enum ObjectPosition {
	Fixed(Rectangle<f64>),
	Relative(Rectangle<f64>)
}

impl Deref for ObjectPosition {
	type Target = Rectangle<f64>;

	fn deref(&self) -> &Self::Target {
		match self {
			ObjectPosition::Fixed(r) | ObjectPosition::Relative(r) => {
				r
			}
		}
	}
}

impl DerefMut for ObjectPosition {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			ObjectPosition::Fixed(r) | ObjectPosition::Relative(r) => {
				r
			}
		}
	}
}

pub enum UiObject {
	Container(container::ContainerState),
	Button(button::ButtonState),
	Input(input::InputState),
}

impl UiObject {
	//
}


pub enum Display {
	None,
	Text(String),
	Render(Box<dyn Fn(&CanvasRenderingContext2d, &Rectangle<f64>) -> Result<()>>)
}


pub enum ObjectUiEvent {
	MouseUp,
	MouseDown,
	MouseClick,
	MouseScroll,

	KeyDown,
	KeyUp
}