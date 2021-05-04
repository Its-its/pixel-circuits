use circuit_sim_common::{config::{EditorType, StateInfo}, http::{AuthedResponse, CanvasJsonReqResp}};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{ProgressEvent, UrlSearchParams, XmlHttpRequest};

use crate::{Error, InnerEditor, NotificationType, Result, config, error::DisplayError, statics::{get_editor_state, get_editor_state_mut, set_editor_state}, window};

use super::{CanvasState, EditorState};



#[derive(Default)]
pub struct InitState {
	pub loading_canvas: Option<CanvasJsonReqResp>,
	pub closure_list_canvi: Option<Closure<dyn FnMut(ProgressEvent)>>,
	pub closure_is_authed: Option<Closure<dyn FnMut(ProgressEvent)>>
}


impl InitState {
	pub fn query_is_authed(&mut self) -> Result<()> {
		let xhr = XmlHttpRequest::new()?;

		let closure = Closure::once(
			move |e: ProgressEvent| {
				let resp = || -> Result<()> {
					let xhr: XmlHttpRequest = e.target().unwrap().dyn_into().unwrap();

					let user = serde_json::from_str::<AuthedResponse>(&xhr.response_text()?.unwrap())??;

					{
						unsafe {
							crate::statics::USER_INFO = Some(user);
						}

						if let Some(state) = get_editor_state_mut().as_any_mut().downcast_mut::<InitState>() {
							state.closure_is_authed = None;
						}
					}

					Self::finish_loading()?;

					Ok(())
				};

				if let Err(e) = resp() {
					log!("Auth Check Error: {:?}", e);
					crate::statics::create_notification("Auth Check", NotificationType::Error(e), 0).expect("Auth Notification");
				}
			}
		);

		xhr.add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())?;

		xhr.open_with_async("GET", "/authed", true)?;

		xhr.send()?;

		self.closure_is_authed = Some(closure);

		Ok(())
	}

	// pub fn query_list_canvi(&mut self, editor: Editor) -> Result<()> {
	// 	let xhr = XmlHttpRequest::new()?;

	// 	let closure = Closure::once(
	// 		move |e: ProgressEvent| {
	// 			let resp = || -> Result<()> {
	// 				let xhr: XmlHttpRequest = e.target().unwrap().dyn_into().unwrap();

	// 				let list = serde_json::from_str::<ListCanviResponse>(&xhr.response_text()?.unwrap())??;

	// 				if let Some(state) = get_editor_state_mut().as_any_mut().downcast_mut::<InitState>() {
	// 					state.closure_list_canvi = None;
	// 				}

	// 				Self::finish_loading(&editor)?;

	// 				Ok(())
	// 			};

	// 			if let Err(e) = resp() {
	// 				log!("List Canvi Error: {:?}", e);
	// 				crate::statics::create_notification("List Canvi", NotificationType::Error(e), 0).expect("List Canvi Notification");
	// 			}
	// 		}
	// 	);

	// 	xhr.add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())?;

	// 	xhr.open_with_async("GET", "/list/canvi", true)?;

	// 	xhr.send()?;

	// 	self.closure_list_canvi = Some(closure);

	// 	Ok(())
	// }

	pub fn finish_loading() -> Result<()> {
		{ // Checks to make sure both closures are None (finished)
			if let Some(state) = get_editor_state().as_any_ref().downcast_ref::<InitState>() {
				if state.closure_is_authed.is_some() || state.closure_list_canvi.is_some() {
					return Ok(());
				}
			}
		}

		if let Err(e) = finish_loading() {
			crate::statics::create_notification("Initialization Error", NotificationType::Error(e), 0)?;
		}

		Ok(())
	}
}




impl EditorState for InitState {
	fn init(&mut self, _: &mut InnerEditor) -> Result<()> {
		self.query_is_authed()?;

		Ok(())
	}

    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_state_info(&self) -> StateInfo {
        unreachable!()
    }

    fn set_state_info(&mut self, _: StateInfo) {
        unreachable!()
    }
}






pub fn finish_loading() -> Result<()> {
	let location = window().location();

	let search = location.search()?;

	if search.is_empty() || search.len() == 1 {
		set_editor_state(Box::new(CanvasState::new()))?;
	} else {
		let params = UrlSearchParams::new_with_str(&search[1..])?;

		// Figure out the current state we're loading into.
		let id_type = match (params.get("canvas"), params.get("object")) {
			(Some(id), None) => Some((id, EditorType::Canvas)),
			(None, Some(id)) => Some((id, EditorType::CustomObject)),

			_ => None
		};

		if let Some((id, type_of)) = id_type {
			if id.is_empty() {
				// Load into a new state.
				set_editor_state(Box::new(
					match type_of {
						EditorType::Canvas => CanvasState::new(),
						EditorType::CustomObject => unimplemented!("Custom Object")
					}
				))?;
			} else {
				// Try loading specified state
				load_and_setup(id)?;
			}
		} else {
			set_editor_state(Box::new(CanvasState::new()))?;
		}
	}

	Ok(())
}

fn load_and_setup(id: String) -> Result<()> {
	log!("load_and_setup: {}", id);

	config::load(id, Box::new(move |result: Option<CanvasJsonReqResp>| {
		if let Some(config) = result {
			let init = get_editor_state_mut().as_any_mut().downcast_mut::<InitState>().ok_or_else::<Error, _>(|| DisplayError::InvalidState.into())?;

			init.loading_canvas = Some(config);

			create_state(init.loading_canvas.take().unwrap())?;
		} else {
			// TODO: Error screen?
		}

		Ok(())
	}))
}

fn create_state(json: CanvasJsonReqResp) -> Result<()> {
	let config = config::create_config_from_json(json)?;

	log!("create_state");

	set_editor_state(Box::new(config))?;

	Ok(())
}