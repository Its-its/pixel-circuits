use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{HtmlDivElement, HtmlElement, HtmlSpanElement, MouseEvent};

use crate::{Result, config::save_editor, editor::CanvasState, statics};


pub struct Topbar {
	container: HtmlDivElement,

	row: HtmlDivElement,
	dropdown: HtmlDivElement
}


impl Topbar {
	pub fn new() -> Self {
		Self {
			row: crate::create_element::<HtmlDivElement>("div"),
			dropdown: crate::create_element::<HtmlDivElement>("div"),
			container: crate::create_element::<HtmlDivElement>("div")
		}
	}

	pub fn init(&self) {
		self.container.set_class_name("editor-ui-top-bar");
		self.row.set_class_name("top-bar-row");
		self.dropdown.set_class_name("top-bar-dropdown");
	}

	pub fn render(&self, parent: &HtmlElement) -> Result<()> {
		parent.append_with_node_1(&self.container)?;
		self.container.append_with_node_1(&self.row)?;
		self.container.append_with_node_1(&self.dropdown)?;

		{ // New
			let container = crate::create_element::<HtmlDivElement>("div");
			container.set_class_name("top-bar-item-container");

			let text_container = crate::create_element::<HtmlSpanElement>("span");
			text_container.set_inner_text("New");
			container.append_with_node_1(&text_container)?;

			self.row.append_with_node_1(&container)?;

			// On click button
			let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
				let _ = statics::set_editor_state(Box::new(CanvasState::new()));

				event.prevent_default();
			}) as Box<dyn FnMut(_)>);
			container.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
			closure.forget();
		}

		{ // Save
			let container = crate::create_element::<HtmlDivElement>("div");
			container.set_class_name("top-bar-item-container");

			let text_container = crate::create_element::<HtmlSpanElement>("span");
			text_container.set_inner_text("Save");
			container.append_with_node_1(&text_container)?;

			self.row.append_with_node_1(&container)?;

			// On click button
			let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
				let res = save_editor(|_| {
					log!("saved");
				});

				if let Err(e) = res {
					log!("Save Error: {:?}", e);
				}

				event.prevent_default();
			}) as Box<dyn FnMut(_)>);
			container.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
			closure.forget();
		}

		Ok(())
	}
}



// pub fn save_popup(editor: Editor) -> Result<()> {
// 	let popup = crate::create_element::<HtmlDivElement>("div");
// 	popup.set_class_name("popup");
// 	body().append_with_node_1(&popup)?;

// 	let popup_inner = crate::create_element::<HtmlDivElement>("div");
// 	popup_inner.set_class_name("popup-container");
// 	popup.append_with_node_1(&popup_inner)?;

// 	let editor_read = editor.read()?;
// 	let canvas = editor_read.canvas()?;
// 	let canvas_state = canvas.read()?;

// 	{ // Canvas ID
// 		let row = crate::create_element::<HtmlDivElement>("div");
// 		row.set_class_name("row");
// 		popup_inner.append_with_node_1(&row)?;

// 		let title = crate::create_element::<HtmlSpanElement>("span");
// 		title.set_class_name("title");
// 		row.append_with_node_1(&title)?;

// 		if let Some(canvas_id) = canvas_state.info.canvas_id.clone() {
// 			title.set_inner_text(&format!("Updating ID: {}", canvas_id));
// 		} else {
// 			title.set_inner_text(&format!("Creating new {}", if editor_read.get_state().custom_object().is_ok() { "Custom Object" } else { "Canvas" }));
// 		}
// 	}

// 	let title_input = {
// 		let row = crate::create_element::<HtmlDivElement>("div");
// 		row.set_class_name("row");
// 		popup_inner.append_with_node_1(&row)?;

// 		let title = crate::create_element::<HtmlSpanElement>("span");
// 		title.set_class_name("title");
// 		title.set_inner_text("Title");
// 		row.append_with_node_1(&title)?;

// 		let input = crate::create_element::<HtmlInputElement>("input");
// 		input.set_class_name("input");
// 		input.set_type("text");
// 		row.append_with_node_1(&input)?;

// 		if let Some(title) = canvas_state.info.title.as_ref() {
// 			input.set_value(title.as_str());
// 		}

// 		input
// 	};

// 	let description_input = {
// 		let row = crate::create_element::<HtmlDivElement>("div");
// 		row.set_class_name("row");
// 		popup_inner.append_with_node_1(&row)?;

// 		let title = crate::create_element::<HtmlSpanElement>("span");
// 		title.set_class_name("title");
// 		title.set_inner_text("Description");
// 		row.append_with_node_1(&title)?;

// 		let input = crate::create_element::<HtmlInputElement>("input");
// 		input.set_class_name("input");
// 		input.set_type("text");
// 		row.append_with_node_1(&input)?;

// 		if let Some(title) = canvas_state.info.description.as_ref() {
// 			input.set_value(title.as_str());
// 		}

// 		input
// 	};

// 	{ // Buttons
// 		let row = crate::create_element::<HtmlDivElement>("div");
// 		row.set_class_name("row");
// 		popup_inner.append_with_node_1(&row)?;

// 		let is_saving_own_canvas = canvas_state.info.user_id == crate::statics::get_user_info().id;

// 		// Save Button
// 		if is_saving_own_canvas {
// 			let button_save = crate::create_element::<HtmlDivElement>("div");
// 			button_save.set_class_name("button");

// 			if canvas_state.info.canvas_id.is_none() {
// 				button_save.set_inner_text("Save");
// 			} else {
// 				button_save.set_inner_text("Update");
// 			}

// 			row.append_with_node_1(&button_save)?;

// 			std::mem::drop(editor_read);

// 			let editor_c = editor.clone();
// 			let title_input = title_input.clone();
// 			let description_input = description_input.clone();
// 			let popup = popup.clone();

// 			let closure = Closure::once(
// 				move || {
// 					{ // Update Title and Description
// 						let canvas = editor_c.read().unwrap().canvas().unwrap();
// 						let mut canvas_state = canvas.write().unwrap();

// 						canvas_state.info.title = Some(title_input.value()).filter(|v| !v.is_empty());
// 						canvas_state.info.description = Some(description_input.value()).filter(|v| !v.is_empty());
// 					}

// 					editor_c.save(Some(move || popup.remove())).unwrap();
// 				}
// 			);

// 			button_save.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
// 			closure.forget();
// 		} else {
// 			std::mem::drop(editor_read);
// 		}

// 		{
// 			let button_fork = crate::create_element::<HtmlDivElement>("div");
// 			button_fork.set_class_name("button");
// 			button_fork.set_inner_text("Fork");

// 			row.append_with_node_1(&button_fork)?;

// 			let closure = Closure::once(
// 				move || {
// 					{ // Update Title and Description
// 						let canvas = editor.read().unwrap().canvas().unwrap();
// 						let mut canvas_state = canvas.write().unwrap();

// 						canvas_state.info.title = Some(title_input.value()).filter(|v| !v.is_empty());
// 						canvas_state.info.description = Some(description_input.value()).filter(|v| !v.is_empty());

// 						canvas_state.info.user_id = crate::statics::get_user_info().id;
// 						canvas_state.info.forked_id = canvas_state.info.canvas_id.clone();
// 						canvas_state.info.canvas_id = None;
// 						canvas_state.info.revision = 0;
// 					}

// 					editor.save(Some(move || popup.remove())).unwrap();
// 				}
// 			);

// 			button_fork.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
// 			closure.forget();
// 		}
// 	}

// 	Ok(())
// }