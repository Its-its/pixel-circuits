use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::Result;
use crate::editor::{Cursor, EditorEvent};
use crate::editor::util::RetangleOpts;
use super::{Display, ObjectPosition, ObjectUiEvent};

// TODO: More Customization
// Font height, font type, center text, align (center, ..) button on pos

pub struct ButtonState {
	position: ObjectPosition,

	display: Display,
	// TODO: Replace with Multi-event callbacks.
	on_click: Box<dyn FnMut()>
}

impl ButtonState {
	pub fn new(display: Display, position: ObjectPosition, on_click: Box<dyn FnMut()>) -> Self {
		Self {
			display,
			on_click,
			position
		}
	}

	pub fn render(&self, cursor: &Cursor, ctx: &CanvasRenderingContext2d) -> Result<()> {
		if self.position.is_cursor_inside(cursor) {
			ctx.set_fill_style(&JsValue::from_str("#222"));
		} else {
			ctx.set_fill_style(&JsValue::from_str("#333"));
		}

		ctx.set_line_width(1.0);

		ctx.translate(self.position.x, self.position.y)?;

		self.position.zero_xy().render_fill(ctx);

		match &self.display {
			Display::Text(text) => {
				ctx.set_font("14px Verdana");
				ctx.set_text_baseline("middle");
				ctx.set_text_align("center");
				ctx.set_fill_style(&JsValue::from_str("#000"));
				ctx.fill_text(text.as_str(), self.position.width / 2.0, self.position.height / 2.0)?;
			}

			Display::Render(func) => {
				func(ctx, &self.position)?
			}

			Display::None => ()
		}

		ctx.translate(-self.position.x, -self.position.y)?;

		Ok(())
	}

	pub fn update(&mut self, cursor: &Cursor, event: &EditorEvent) -> Result<Option<ObjectUiEvent>> {
		if self.position.is_cursor_inside(cursor) {
			if let EditorEvent::MouseUp(_) = event {
				(self.on_click)();

				return Ok(Some(ObjectUiEvent::MouseUp));
			}
		}

		Ok(None)
	}
}