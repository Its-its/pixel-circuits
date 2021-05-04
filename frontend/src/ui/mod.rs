use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, HtmlDivElement, Event};

use crate::error::{Result, EditingError};
use crate::{Editor, window};

mod sidebar;
mod topbar;
pub mod notification;

use sidebar::Sidebar;
use topbar::Topbar;

pub use notification::{NotificationType, Notification};

pub use sidebar::ItemContainer;

#[derive(Clone)]
pub struct MainUi(Rc<RwLock<InnerUI>>);


impl MainUi {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Result<Self> {
		Ok(Self(Rc::new(RwLock::new(InnerUI::new()?))))
	}


	pub fn read(&self) -> Result<RwLockReadGuard<'_, InnerUI>> {
		self.0.read().map_err(|_| EditingError::PoisonError.into())
	}

	pub fn write(&self) -> Result<RwLockWriteGuard<'_, InnerUI>> {
		self.0.write().map_err(|_| EditingError::PoisonError.into())
	}


	pub fn init(&self) -> Result<()> {
		{
			let mut inner = self.write()?;
			inner.init(self.clone())?;
		}

		{ // On Window Resize
			let inner = self.clone();
			let closure = Closure::wrap(Box::new(move |_event: Event| {
				let inner = inner.read().unwrap();
				let mut editor = inner.editor.write().unwrap();

				let size = (inner.container_editor.client_width() as usize, inner.container_editor.client_height() as usize);

				editor.resize(size.0, size.1);
			}) as Box<dyn FnMut(_)>);

			window().add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;

			closure.forget();
		}

		Ok(())
	}

	pub fn render(&self, parent: &HtmlElement) -> Result<()> {
		self.write()?.render(parent)
	}

	pub fn run(&self) -> Result<()> {
		let mut inner = self.write()?;

		{ // On Window Load
			let mut editor = inner.editor.write()?;

			let size = (inner.container_editor.client_width() as usize, inner.container_editor.client_height() as usize);

			editor.resize(size.0, size.1);
		}

		inner.run();

		Ok(())
	}
}


pub struct InnerUI {
	container: HtmlDivElement,
	container_bottom: HtmlDivElement,

	pub sidebar: Sidebar,
	pub topbar: Topbar,
	pub editor: Editor,

	container_editor: HtmlDivElement,
}


impl InnerUI {
	pub fn new() -> Result<Self> {
		let container = crate::create_element::<HtmlDivElement>("div");
		let container_bottom = crate::create_element::<HtmlDivElement>("div");
		let container_editor = crate::create_element::<HtmlDivElement>("div");

		let editor = Editor::new(500, 500);

		let sidebar = Sidebar::new(editor.clone())?;
		let topbar = Topbar::new();

		Ok(Self {
			container,
			container_bottom,
			container_editor,

			sidebar,
			topbar,
			editor
		})
	}

	pub fn init(&mut self, ui: MainUi) -> Result<()> {
		self.container.set_class_name("editor-ui-container");
		self.container_bottom.set_class_name("editor-ui-bottom");
		self.container_editor.set_class_name("editor-ui-editor-container");

		self.topbar.init();
		self.sidebar.init()?;

		self.editor.write()?.main_ui = Some(ui);

		Ok(())
	}

	pub fn render(&mut self, parent: &HtmlElement) -> Result<()> {
		parent.append_with_node_1(&self.container)?;

		self.topbar.render(&self.container)?;
		self.container.append_with_node_1(&self.container_bottom)?;

		self.sidebar.render(&self.container_bottom)?;

		self.container_bottom.append_with_node_1(&self.container_editor)?;

		self.editor.init(&self.container_editor)?;

		Ok(())
	}

	pub fn run(&mut self) {
		self.editor.render();
	}
}