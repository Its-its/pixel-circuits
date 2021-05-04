use std::ops::Deref;

use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Node};

use wasm_bindgen::JsCast;


mod pixels;
pub use pixels::*;



#[derive(Debug, Clone)]
pub struct Canvas {
	pub element: HtmlCanvasElement,
	pub context: CanvasRenderingContext2d
}


impl Canvas {
	pub fn new() -> Self {
		let element = crate::create_element::<HtmlCanvasElement>("canvas");

		let context = element
			.get_context("2d")
			.unwrap()
			.unwrap()
			.dyn_into::<CanvasRenderingContext2d>()
			.unwrap();

		Self {
			element,
			context
		}
	}

	pub fn set_width(&self, value: usize) {
		self.element.set_width(value as u32);
	}

	pub fn set_height(&self, value: usize) {
		self.element.set_height(value as u32);
	}
}


impl AsRef<HtmlCanvasElement> for Canvas {
	fn as_ref(&self) -> &HtmlCanvasElement {
		&self.element
	}
}

impl AsRef<Node> for Canvas {
	fn as_ref(&self) -> &Node {
		&self.element
	}
}

impl Deref for Canvas {
	type Target = CanvasRenderingContext2d;

	fn deref(&self) -> &Self::Target {
		&self.context
	}
}