use circuit_sim_common::object::ObjectType;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{HtmlDivElement, HtmlElement, HtmlLiElement, HtmlUListElement, MouseEvent};


use crate::{Editor, Error, NotificationType, Result, canvas::{
		PIXEL_ODD,
		PixelColor
	}, editor::{
		CanvasState,
		state::canvas::{
			CanvasEvent,
			CanvasTool
		}
	}, error::DisplayError, objects::create_new_object_from_type, statics::{get_editor_state, get_editor_state_mut}};

mod item_container;
mod start;

pub use item_container::ItemContainer;
use start::StartContainer;

pub struct Sidebar {
	pub container: HtmlDivElement,
	pub top: HtmlDivElement,
	pub bottom: HtmlDivElement,

	pub editor: Editor,

	pub item_containers: Vec<ItemContainer>,
	pub obj_settings: Option<ItemContainer>,

	pub start_container: StartContainer
}


impl Sidebar {
	pub fn new(editor: Editor) -> Result<Self> {
		let top = crate::create_element::<HtmlDivElement>("div");

		Ok(Self {
			obj_settings: None,
			item_containers: Vec::new(),
			start_container: StartContainer::new(),
			container: crate::create_element::<HtmlDivElement>("div"),
			bottom: crate::create_element::<HtmlDivElement>("div"),
			top,
			editor
		})
	}

	pub fn render_all(&self) {
		self.item_containers.iter().for_each(|i| { let _ = i.render(); });
	}

	pub fn init_item_containers(&mut self) -> Result<()> {
		self.item_containers.append(&mut vec![
			create_tools_container(self.top.clone())?,
			create_pixels_container(self.top.clone())?,
			create_objects_container(self.top.clone())?
		]);

		Ok(())
	}

	pub fn init(&mut self) -> Result<()> {
		self.container.set_class_name("editor-ui-side-bar");
		self.top.set_class_name("editor-ui-side-bar-top");
		self.bottom.set_class_name("editor-ui-side-bar-bottom");

		Ok(())
	}

	pub fn render(&mut self, parent: &HtmlElement) -> Result<()> {
		parent.append_with_node_1(&self.container)?;
		self.container.append_with_node_1(&self.top)?;
		self.container.append_with_node_1(&self.bottom)?;

		self.start_container.render(&self.bottom)?;

		Ok(())
	}
}

pub fn create_tools_container(top: HtmlDivElement) -> Result<ItemContainer> {
	let cont = ItemContainer::new("Tools", "connection-colors", top)?;

	let item_list = crate::create_element::<HtmlUListElement>("ul");
	item_list.set_class_name("item-list");
	cont.container_inner.append_with_node_1(&item_list)?;

	let items = vec![
		(PixelColor(237, 174, 192), CanvasTool::Eraser)
	];

	for (pixel, tool) in items {
		let list_item = crate::create_element::<HtmlLiElement>("li");
		list_item.set_class_name("item");

		item_list.append_with_node_1(&list_item)?;

		let image = crate::create_element::<HtmlDivElement>("div");
		image.set_attribute(
			"style",
			&format!(
				"background-color: {}",
				pixel.get_string_color()
			)
		)?;
		list_item.append_with_node_1(&image)?;

		{ // Mouse Up Event
			let closure = Closure::wrap(Box::new(move |e: MouseEvent| {
				if e.button() != 0 {
					return;
				}

				let run = || -> Result<()> {
					let editor = get_editor_state_mut();

					if let Some(state) = editor.as_any_mut().downcast_mut::<CanvasState>() {
						state.event = Some(CanvasEvent::Tooling(tool));
					}

					Ok(())
				};

				if let Err(e) = run() {
					log!("Side Bar Error Occured: {:?}", e);
					crate::statics::create_notification("Create Object", NotificationType::Error(e), 1000 * 10).expect("notification");
				}

			}) as Box<dyn FnMut(_)>);
			list_item.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
			closure.forget();
		}
	}

	Ok(cont)
}


pub fn create_pixels_container(top: HtmlDivElement) -> Result<ItemContainer> {
	let cont = ItemContainer::new("Connections", "connection-colors", top)?;

	let item_list = crate::create_element::<HtmlUListElement>("ul");
	item_list.set_class_name("item-list");
	cont.container_inner.append_with_node_1(&item_list)?;

	let editor = get_editor_state();
	let editor = editor.get_canvas_state().ok_or_else::<Error, _>(|| DisplayError::InvalidState.into())?;

	for (pixel_color, _) in editor.pixels.palette.clone() {
		let list_item = crate::create_element::<HtmlLiElement>("li");
		list_item.set_class_name("item");

		item_list.append_with_node_1(&list_item)?;

		let image = crate::create_element::<HtmlDivElement>("div");
		image.set_attribute(
			"style",
			&format!(
				"background-color: {}",
				pixel_color.get_string_color()
			)
		)?;
		list_item.append_with_node_1(&image)?;

		{ // Mouse Up Event
			let closure = Closure::wrap(Box::new(move |e: MouseEvent| {
				if e.button() != 0 {
					return;
				}

				let run = || -> Result<()> {
					let editor = get_editor_state_mut();

					if let Some(state) = editor.as_any_mut().downcast_mut::<CanvasState>() {
						state.event = Some(CanvasEvent::Painting(pixel_color));
					}

					Ok(())
				};

				if let Err(e) = run() {
					log!("Side Bar Error Occured: {:?}", e);
					crate::statics::create_notification("Create Object", NotificationType::Error(e), 1000 * 10).expect("notification");
				}

			}) as Box<dyn FnMut(_)>);
			list_item.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
			closure.forget();
		}
	}

	Ok(cont)
}


pub fn create_objects_container(top: HtmlDivElement) -> Result<ItemContainer> {
	let cont = ItemContainer::new("Objects", "connection-colors", top)?;

	let item_list = crate::create_element::<HtmlUListElement>("ul");
	item_list.set_class_name("item-list");
	cont.container_inner.append_with_node_1(&item_list)?;

	for obj_type in ObjectType::list() {
		let list_item = crate::create_element::<HtmlLiElement>("li");
		list_item.set_class_name("item");

		item_list.append_with_node_1(&list_item)?;

		let image = crate::create_element::<HtmlDivElement>("div");
		image.set_attribute(
			"style",
			&format!(
				"background-color: {}",
				PIXEL_ODD.get_string_color()
			)
		)?;
		list_item.append_with_node_1(&image)?;

		{ // Mouse Up Event
			let closure = Closure::wrap(Box::new(move |e: MouseEvent| {
				if e.button() != 0 {
					return;
				}

				let run = || -> Result<()> {
					let editor = get_editor_state_mut();

					if let Some(state) = editor.as_any_mut().downcast_mut::<CanvasState>() {
						let mut new_obj = create_new_object_from_type(obj_type);
						new_obj.set_cell_pos((10, 10));

						state.event = Some(CanvasEvent::MovingObject(new_obj.get_id()));
						state.pixels.add_object(new_obj);
					}

					Ok(())
				};

				if let Err(e) = run() {
					log!("Side Bar Error Occured: {:?}", e);
					crate::statics::create_notification("Create Object", NotificationType::Error(e), 1000 * 10).expect("notification");
				}

			}) as Box<dyn FnMut(_)>);
			list_item.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
			closure.forget();
		}
	}

	Ok(cont)
}

// pub fn create_vertical_list_container(top: HtmlDivElement, editor: Editor) -> Result<ItemContainer> {
// 	let cont = ItemContainer::new("Other", "item-vertical", top)?;

// 	let item_list = crate::create_element::<HtmlUListElement>("ul");
// 	item_list.set_class_name("item-list");
// 	cont.container_inner.append_with_node_1(&item_list)?;

// 	{ // New Line
// 		let list_item = crate::create_element::<HtmlLiElement>("li");
// 		list_item.set_class_name("item");
// 		list_item.set_inner_text("Create Line");
// 		item_list.append_with_node_1(&list_item)?;

// 		// On click button
// 		let editor = editor.clone();
// 		let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
// 			if let Ok(canvas) = editor.write().unwrap().canvas() {
// 				canvas.write().unwrap().editing = EditingType::CreateConnection;
// 			}

// 			event.prevent_default();
// 		}) as Box<dyn FnMut(_)>);
// 		item_list.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
// 		closure.forget();
// 	}

// 	{ // Create Text
// 		let list_item = crate::create_element::<HtmlLiElement>("li");
// 		list_item.set_class_name("item");
// 		list_item.set_inner_text("Create Text Label");
// 		item_list.append_with_node_1(&list_item)?;

// 		// On click button
// 		let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
// 			if let Ok(canvas) = editor.write().unwrap().canvas() {
// 				let mut write = canvas.write().unwrap();

// 				let text = Text::new("New Text".to_string(), 100.0, 100.0);
// 				write.editing = EditingType::Text(text.id, TextEvent::Placing);
// 				write.text_objects.push(text);
// 			}

// 			event.prevent_default();
// 		}) as Box<dyn FnMut(_)>);
// 		item_list.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
// 		closure.forget();
// 	}


// 	Ok(cont)
// }