use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{
	HtmlDivElement, HtmlSpanElement,
	MouseEvent
};

use crate::Result;



pub struct ItemContainer {
	// Elements
	pub container: HtmlDivElement,
	pub container_topbar: HtmlDivElement,
	pub topbar_min_max: HtmlDivElement,
	pub topbar_title_cont: HtmlDivElement,
	pub topbar_title: HtmlSpanElement,
	pub container_inner: HtmlDivElement,

	parent: HtmlDivElement
}

impl ItemContainer {
	pub fn new(title: &str, clazz: &str, parent: HtmlDivElement) -> Result<Self> {
		let container = crate::create_element::<HtmlDivElement>("div");
		container.set_class_name(&(String::from("item-container ") + clazz));

		// Topbar
		let container_topbar = crate::create_element::<HtmlDivElement>("div");
		container_topbar.set_class_name("container-topbar");
		container.append_with_node_1(&container_topbar)?;

		// Minimize / Maximaize
		let topbar_min_max = crate::create_element::<HtmlDivElement>("div");
		topbar_min_max.set_class_name("topbar-min-max");
		container_topbar.append_with_node_1(&topbar_min_max)?;

		{ // On click
			let element_copied = container.clone();
			let closure = Closure::wrap(Box::new(move |_: MouseEvent| {
				let list = element_copied.class_list();

				if list.contains("minimized") {
					let _ = list.remove_1("minimized");
				} else {
					let _ = list.add_1("minimized");
				}
			}) as Box<dyn FnMut(_)>);
			topbar_min_max.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
			closure.forget();
		}

		// Title
		let topbar_title_cont = crate::create_element::<HtmlDivElement>("div");
		topbar_title_cont.set_class_name("topbar-title");
		container_topbar.append_with_node_1(&topbar_title_cont)?;

		let topbar_title = crate::create_element::<HtmlSpanElement>("span");
		topbar_title.set_inner_text(title);
		topbar_title_cont.append_with_node_1(&topbar_title)?;

		// Inner
		let container_inner = crate::create_element::<HtmlDivElement>("div");
		container_inner.set_class_name("container-inner");
		container.append_with_node_1(&container_inner)?;

		Ok(ItemContainer {
			container,
			container_topbar,
			topbar_min_max,
			topbar_title_cont,
			topbar_title,
			container_inner,

			parent
		})
	}

	pub fn render(&self) -> Result<()> {
		if self.container.parent_element().is_none() {
			self.parent.append_with_node_1(&self.container)?;
		}

		Ok(())
	}

	pub fn is_minimized(&self) -> bool {
		self.container.class_list().contains("minimized")
	}

	pub fn set_minimized(&self, value: bool) {
		let list = self.container.class_list();

		if self.is_minimized() != value {
			if value {
				let _ = list.add_1("minimized");
			} else {
				let _ = list.remove_1("minimized");
			}
		}
	}

	pub fn remove(&self) {
		self.container.remove();
	}
}