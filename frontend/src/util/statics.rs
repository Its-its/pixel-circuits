pub use user::*;
pub use running::*;
pub use notifications::*;
pub use ticking::*;
pub use state::*;
pub use ui::*;

pub mod state {
	use crate::Result;
    use crate::editor::EditorState;

    use super::get_ui;

	pub static mut EDITOR_STATE: Option<Box<dyn EditorState>> = None;

	pub fn get_editor_state<'a>() -> &'a dyn EditorState {
		unsafe { &**EDITOR_STATE.as_ref().unwrap() }
	}

	pub fn get_editor_state_mut<'a>() -> &'a mut dyn EditorState {
		unsafe { &mut **EDITOR_STATE.as_mut().unwrap() }
	}

	pub fn set_editor_state(state: Box<dyn EditorState>) -> Result<()> {
		replace_editor_state(state)?;
		Ok(())
	}

	pub fn replace_editor_state(state: Box<dyn EditorState>) -> Result<Box<dyn EditorState>> {
		let old_state = std::mem::replace(unsafe { &mut *EDITOR_STATE.as_mut().unwrap() }, state);

		// TODO: Might wanna 1ms timeout this.
		{
			let editor = {
				let ui = get_ui();
				let ui = ui.read()?;
				ui.editor.clone()
			};

			let mut editor = editor.write()?;

			get_editor_state_mut().init(&mut *editor)?;
		}

		Ok(old_state)
	}
}

pub mod ui {
	use crate::MainUi;

	pub static mut MAIN_UI: Option<MainUi> = None;

	pub fn get_ui() -> MainUi {
		unsafe { MAIN_UI.clone().unwrap() }
	}
}

// USER INFO
pub mod user {
	use circuit_sim_common::http::UserInfo;

	pub static mut USER_INFO: Option<UserInfo> = None;

	pub fn get_user_info<'a>() -> &'a UserInfo {
		unsafe { USER_INFO.as_ref().unwrap() }
	}

	pub fn has_user_info() -> bool {
		unsafe { USER_INFO.is_some() }
	}
}


// NOTIFICATION MANAGER
pub mod notifications {
	use crate::{NotificationType, Result};
	use crate::ui::notification::{
		NotificationManager,
		Notification
	};

	pub static mut NOTIFICATION_MANAGER: Option<NotificationManager> = None;

	pub fn remove_notification(rendered_at: f64) {
		unsafe {
			NOTIFICATION_MANAGER.as_mut().unwrap().remove(rendered_at);
		}
	}

	pub fn display_notification(notification: Notification) -> Result<()> {
		unsafe { NOTIFICATION_MANAGER.as_mut().unwrap().display(notification) }
	}

	pub fn create_notification<T: Into<String>>(title: T, notif_type: NotificationType, display_time: i32) -> Result<()> {
		let mut notification = Notification::new(title.into(), notif_type);
		notification.set_display_time(display_time);

		display_notification(notification)
	}
}


// Editor Running?
pub mod running {
	use crate::body;

	static mut IS_EDITOR_RUNNING: bool = false;

	pub fn is_editor_running() -> bool {
		unsafe { IS_EDITOR_RUNNING }
	}

	pub fn set_editor_running(value: bool) {
		if value {
			let _ = body().class_list().add_1("editor-running");
		} else {
			let _ = body().class_list().remove_1("editor-running");
		}

		unsafe { IS_EDITOR_RUNNING = value; }
	}
}


// How the editor ticking works.
pub mod ticking {
	use std::mem;

	use crate::Result;
	use crate::objects::ObjectData;

	pub static mut EDITOR_TICKING: Ticker = Ticker::new();

	pub struct Ticker {
		queries: Vec<ObjectData>
	}

	impl Ticker {
		pub const fn new() -> Self {
			Self {
				queries: Vec::new()
			}
		}

		/// Remove duplicate `ObjectData`'s.
		pub fn remove_duplicates(&mut self) {
			self.queries = mem::take(&mut self.queries)
				.into_iter()
				.fold(Vec::new(), |mut unqiue, data| {
					if !unqiue.contains(&data) {
						unqiue.push(data);
					}

					unqiue
				});
		}

		pub fn tick_all(&mut self) -> Result<()> {
			if let Some(canvas_state) = super::get_editor_state_mut().get_canvas_state_mut() {
				for data in mem::take(&mut self.queries) {
					if let Some(data) = data.continue_sending(canvas_state) {
						self.queries.append(&mut data?);
					}
				}
			}

			Ok(())
		}
	}

	pub fn add_to_ticking(mut data: Vec<ObjectData>) {
		unsafe {
			EDITOR_TICKING.queries.append(&mut data);
			EDITOR_TICKING.remove_duplicates();
		}
	}

	pub fn tick() -> Result<()> {
		unsafe {
			EDITOR_TICKING.tick_all()
		}
	}
}