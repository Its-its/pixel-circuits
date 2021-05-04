#![warn(warnings, rust_2018_idioms, dead_code)]
#![warn(clippy::pedantic)]
#![allow(
	unsafe_code,

	clippy::clippy::default_trait_access,
	clippy::new_without_default,
	clippy::needless_pass_by_value,
	clippy::collapsible_if,
	clippy::find_map,
	clippy::map_err_ignore,
	clippy::implicit_hasher,
	clippy::match_wildcard_for_single_variants,
	clippy::missing_docs_in_private_items,
	clippy::must_use_candidate,
	clippy::missing_inline_in_public_items,
	clippy::missing_errors_doc,

	clippy::unwrap_used,
	clippy::expect_used,
	clippy::too_many_lines,
	clippy::module_name_repetitions,
	clippy::struct_excessive_bools,
	clippy::similar_names,
	clippy::cast_lossless,
	clippy::cast_possible_truncation,
	clippy::cast_precision_loss,
	clippy::cast_possible_wrap,
	clippy::cast_sign_loss
)]

use editor::state::init::InitState;
use util::statics::EDITOR_STATE;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Window, Document, HtmlElement};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'static> = wee_alloc::WeeAlloc::INIT;


#[macro_use]
mod macros;

pub mod config;
pub mod canvas;
pub mod ui;
pub mod objects;
pub mod editor;
pub mod render_util;
pub mod error;
pub mod util;

pub use error::{Error, Result};
pub use editor::{Editor, InnerEditor, EditorEvent};
pub use ui::{MainUi, Notification, NotificationType};
pub use util::*;

pub use circuit_sim_common::{CellPos, CanvasPos, MARGIN_SIZE};

use ui::notification::NotificationManager;


pub fn create_element<I: wasm_bindgen::JsCast>(element_name: &str) -> I {
	document()
	.create_element(element_name)
	.unwrap()
	.dyn_into::<I>()
	.map_err(|_| ())
	.unwrap()
}


/// Returns Window. Panics if can't.
pub fn window() -> Window {
	web_sys::window().expect("Unwrapping Window")
}

/// Returns Document. Panics if can't.
pub fn document() -> Document {
	window().document().expect("Unwrapping Document")
}

/// Returns Body Element. Panics if can't.
pub fn body() -> HtmlElement {
	document().body().expect("Unwrapping Body")
}



#[wasm_bindgen(start)]
pub fn main() -> Result<()> {
	#[cfg(debug_assertions)]
	console_error_panic_hook::set_once();

	unsafe {
		// Initiate EDITOR_STATE
		EDITOR_STATE = Some(Box::new(InitState::default()));

		// Initiate Notification Manager
		let manager = NotificationManager::new();
		manager.init()?;

		crate::statics::NOTIFICATION_MANAGER = Some(manager);
	}


	let main_ui = MainUi::new()?;
	unsafe { statics::MAIN_UI = Some(main_ui.clone()); }

	if let Err(e) = star_ui(&main_ui) {
		log!("{:?}", e);

		crate::statics::create_notification("Initiating UI", NotificationType::Error(e), 0)?;

		return Ok(());
	}



	Ok(())
}

fn star_ui(ui: &MainUi) -> Result<()> {
	ui.init()?;

	ui.render(&body())?;

	ui.run()?;

	Ok(())
}