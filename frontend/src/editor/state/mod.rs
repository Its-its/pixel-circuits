use std::any::Any;

use crate::{EditorEvent, Result, statics::get_editor_state, window};
use super::InnerEditor;

pub mod canvas;
pub mod init;

pub use canvas::CanvasState;
use circuit_sim_common::config::StateInfo;
use wasm_bindgen::JsValue;


pub trait EditorState {
	fn init(&mut self, editor: &mut InnerEditor) -> Result<()> {
		// Init Sidebar items
		if let Some(ui) = editor.main_ui.clone() {
			ui.read()?.sidebar.item_containers.iter().for_each(|c| {let _ = c.render();});
		}

		Ok(())
	}

	fn render(&self, _editor: &InnerEditor) -> Result<()> {
		Ok(())
	}

	fn update(&mut self, _editor: &mut InnerEditor, _editor_event: &EditorEvent) -> Result<()> {
		Ok(())
	}

	fn search_name(&self) -> &str {
		"canvas"
	}

	fn as_any_ref(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;


	fn get_canvas_state(&self) -> Option<&CanvasState> {
		self.as_any_ref().downcast_ref()
	}

	fn get_canvas_state_mut(&mut self) -> Option<&mut CanvasState> {
		self.as_any_mut().downcast_mut()
	}

	fn get_state_info(&self) -> StateInfo;
	fn set_state_info(&mut self, value: StateInfo);
}


pub fn update_window_location() -> Result<()> {
	let state = get_editor_state();

	let info = state.get_state_info();

	let location = window().location();

	let _ = window().history()?.replace_state_with_url(
		&JsValue::NULL,
		"",
		Some(&format!("{}?{}={}", location.pathname()?, state.search_name(), info.canvas_id.unwrap()))
	);

	Ok(())
}