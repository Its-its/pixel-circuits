// Saving / Loading
// Custom Objects / Editors

use std::collections::HashMap;

use web_sys::{XmlHttpRequest, ProgressEvent};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use circuit_sim_common::{config::{CanvasStateJson, ConfigJson, ObjectNormalJson, ObjectsJson, StateInfo}, http::{CanvasJsonReqResp, LoadResponse, SaveResponse}, object::NodeValue};

use crate::{NotificationType, Result, canvas::{PixelBackground, PixelType}, editor::state::update_window_location, ids::ObjectId};
use crate::error::ConfigError;
use crate::statics::create_notification;
use crate::editor::{CanvasState};


pub struct Config {
	pub info: StateInfo,
	pub editor: CanvasState
}

// Saving into JSON

pub fn save_editor<F>(callback: F) -> Result<()> where F: Fn(bool) + 'static {
	let canvas_editor = crate::statics::get_editor_state();

	let json = CanvasJsonReqResp {
		info: canvas_editor.get_state_info(),
		json: ConfigJson::new(try_from_editor_state(canvas_editor.get_canvas_state().unwrap())?)
	};

	let value = serde_json::to_string(&json)?;

	let xhr = XmlHttpRequest::new().unwrap();

	{
		let closure = Closure::once(
			move |e: ProgressEvent| {
				let xhr: XmlHttpRequest = e.target().unwrap().dyn_into().unwrap();

				let process = move || -> Result<()> {
					let state_info = serde_json::from_str::<SaveResponse>(&xhr.response_text()?.unwrap())??;
					crate::statics::get_editor_state_mut().set_state_info(state_info);

					update_window_location()?;

					Ok(())
				};


				if let Err(e) = process() {
					create_notification("Save Error", NotificationType::Error(e), 0).expect("Load Notification");
					callback(false)
				} else {
					callback(true)
				}
			}
		);

		xhr.add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())?;
		closure.forget();
	}


	xhr.open_with_async("POST", "/save", true)?;
	xhr.set_request_header("Content-type", "application/json")?;

	xhr.send_with_opt_str(Some(value.as_str()))?;

	Ok(())
}


pub fn try_from_editor_state(value: &CanvasState) -> Result<CanvasStateJson> {
	let mut pixels: HashMap<usize, Vec<_>> = HashMap::new();

	for (&pos, pixel) in &value.pixels.cells {
		if let PixelType::Wire { index, .. } = &pixel.type_of {
			pixels.entry(*index).or_default().push(pos);
		}
	}

	Ok(CanvasStateJson {
		pixels,

		objects: value.pixels.objects.iter().map(|v| {
			ObjectsJson::Normal(ObjectNormalJson {
				id: v.get_id().into(),
				type_of: v.get_object_state().type_of,
				pos: v.get_cell_pos(),
				dim: v.get_dimensions(),
				nodes: v.get_object_state().nodes.iter().map(|n| n.into()).collect()
			})
		}).collect(),
		text_objects: Vec::new(),

		color_palette: value.pixels.palette.iter()
			.copied()
			.fold(
				Vec::new(),
				|mut vec, (p1, p2)| {
					vec.push(p1.into());
					vec.push(p2.into());

					vec
				}
			)
	})
}




// Loading from JSON


pub fn load<F>(id: String, callback: F) -> Result<()> where F: Fn(Option<CanvasJsonReqResp>) -> Result<()> + 'static {
	let xhr = XmlHttpRequest::new().unwrap();

	{
		let closure = Closure::once(move |e: ProgressEvent| {
			let resp = |callback: &F| -> Result<()> {
				let xhr: XmlHttpRequest = e.target().unwrap().dyn_into().unwrap();

				let status = xhr.status()?;

				if status == 200 {
					callback(Some(serde_json::from_str::<LoadResponse>(&xhr.response_text()?.unwrap())??))?;

					Ok(())
				} else {
					Err(ConfigError::LoadingError(status, xhr.response().ok().and_then(|v| v.as_string()).unwrap_or_default()).into())
				}
			};

			if let Err(e) = resp(&callback) {
				create_notification("Load Error", NotificationType::Error(e), 0).expect("Load Notification");
				let _ = callback(None);
			}
		});

		xhr.add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())?;
		closure.forget();
	}

	xhr.open_with_async("GET", &format!("/canvas/{}", id), true)?;
	xhr.set_request_header("Content-type", "application/json")?;

	xhr.send()?;

	Ok(())
}

pub fn create_config_from_json(json: CanvasJsonReqResp) -> Result<CanvasState> {
	create_state_from_json(json.json.into_inner_json(), json.info)
}

pub fn create_state_from_json(editor: CanvasStateJson, info: StateInfo) -> Result<CanvasState> {
	let palette = editor.color_palette.chunks(2).map(|val| (val[0].into(), val[1].into())).collect();

	let objects = editor.objects.into_iter()
		.map(|o| {
			match o {
				ObjectsJson::Normal(json) => {
					let mut object = crate::objects::create_new_object(ObjectId::gen_id(), json.type_of, json.pos, Some(json.dim));
					object.get_object_state_mut().nodes = json.nodes.into_iter().map(|node| node.into()).collect();
					object
				}

				_ => unimplemented!()
			}
		})
		.collect();

	let mut background = PixelBackground {
		cells: HashMap::new(),
		palette,
		objects
	};


	for (index, poses) in editor.pixels {
		for pos in poses {
			background.insert_cell(pos, PixelType::Wire { index, value: NodeValue::Gpio(false) });
		}
	}



	background.re_render_objects();


	Ok(CanvasState {
		info,
		event: None,
		pixels: background
	})
}