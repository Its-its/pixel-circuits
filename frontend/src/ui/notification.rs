use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use js_sys::Date;
use web_sys::{HtmlDivElement, HtmlSpanElement};

use crate::{Error, Result, body, window};

const DEFAULT_DISPLAY_TIME: i32 = 1000 * 8;


#[derive(Debug)]
pub enum NotificationType {
	ErrorStr(String),
	Error(Error),
	Info(String),
	Success(String)
}

impl NotificationType {
	pub fn cause(&self) -> &str {
		if matches!(self, NotificationType::Error(_)) {
			if matches!(self, NotificationType::Error(Error::JsValue(_))) {
				"Javascript"
			} else {
				"WASM"
			}
		} else {
			"Manual"
		}
	}
}


pub struct NotificationManager {
	container: HtmlDivElement,
	notifications: Vec<Notification>
}

impl NotificationManager {
	pub fn new() -> Self {
		NotificationManager {
			container: crate::create_element::<HtmlDivElement>("div"),
			notifications: Vec::new()
		}
	}

	pub fn init(&self) -> Result<()> {
		self.container.set_class_name("notification-container");

		Ok(())
	}

	pub fn display(&mut self, mut notification: Notification) -> Result<()> {
		notification.render(self.get_or_create_container()?)?;

		self.notifications.push(notification);

		Ok(())
	}

	pub fn get_or_create_container(&self) -> Result<&HtmlDivElement> {
		if self.container.parent_element().is_none() {
			body().append_with_node_1(&self.container)?;
		}

		Ok(&self.container)
	}

	pub fn remove(&mut self, rendered_at: f64) {
		if let Some(index) = self.notifications.iter().position(|n| (n.rendered_at - rendered_at).abs() < 0.001) {
			self.notifications.remove(index);

			if self.container.first_element_child().is_none() {
				self.container.remove();
			}
		}
	}
}

pub struct Notification {
	title: String,
	notif_type: NotificationType,

	rendered_at: f64,
	display_time: i32,

	container: HtmlDivElement,

	timeout: Option<i32>,
	listeners: Vec<Closure<dyn FnMut()>>
}

impl Notification {
	pub fn new(title: String, notif_type: NotificationType) -> Self {
		Notification {
			title,
			notif_type,

			rendered_at: Date::now(),
			display_time: DEFAULT_DISPLAY_TIME,

			container: crate::create_element::<HtmlDivElement>("div"),

			timeout: None,
			listeners: Vec::new()
		}
	}

	pub fn set_display_time(&mut self, value: i32) {
		self.display_time = value;
	}

	pub fn render(&mut self, container_body: &HtmlDivElement) -> Result<()> {
		let rendered_at = self.rendered_at;

		while let Some(child) = self.container.first_element_child().as_ref() {
			child.remove();
		}

		let notif_type = match self.notif_type {
			NotificationType::Success(_) => "success",
			NotificationType::Info(_) => "info",
			NotificationType::Error(_) | NotificationType::ErrorStr(_) => "error"
		};

		self.container.set_class_name(&format!("notification {}", notif_type));


		// Title Bar
		{
			let inner = crate::create_element::<HtmlDivElement>("div");
			inner.set_class_name("flex-row title-bar");

			// Title
			{
				let span = crate::create_element::<HtmlSpanElement>("span");
				span.set_class_name("title");
				span.set_inner_text(&self.title);
				inner.append_with_node_1(&span)?;
			}

			// Button
			{
				let span = crate::create_element::<HtmlSpanElement>("span");
				span.set_class_name("close-button");
				span.set_inner_text("X");
				inner.append_with_node_1(&span)?;

				// TODO: Error when clicking to close.
				{ // On button click - Close
					let closure = Closure::once(move || crate::statics::remove_notification(rendered_at));

					span.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;

					self.listeners.push(closure);
				}
			}

			self.container.append_with_node_1(&inner)?;
		}

		// Expand Info
		{
			let inner = crate::create_element::<HtmlDivElement>("div");
			inner.set_class_name("flex-column expansion");

			{ // On Container click
				let inner = inner.clone();

				let closure = Closure::wrap(Box::new(move || {
					if inner.class_list().contains("show") {
						let _ = inner.class_list().remove_1("show");
					} else {
						let _ = inner.class_list().add_1("show");
					}
				}) as Box<dyn FnMut()>);

				self.container.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;

				self.listeners.push(closure);
			}

			// Description
			{
				let span = crate::create_element::<HtmlSpanElement>("span");
				span.set_class_name("desc");
				span.set_inner_text(&format!("{:?}", &self.notif_type));
				inner.append_with_node_1(&span)?;
			}

			// Cause
			{
				let span = crate::create_element::<HtmlSpanElement>("span");
				span.set_class_name("cause");
				span.set_inner_text(self.notif_type.cause());
				inner.append_with_node_1(&span)?;
			}

			// Cause
			{
				let span = crate::create_element::<HtmlSpanElement>("span");
				span.set_class_name("creation");
				span.set_inner_text(&Date::new(&JsValue::from(self.rendered_at)).to_locale_time_string("en-US").as_string().unwrap());
				inner.append_with_node_1(&span)?;
			}

			self.container.append_with_node_1(&inner)?;
		}

		if let Some(timeout) = self.timeout {
			window().clear_timeout_with_handle(timeout);
		} else {
			container_body.append_with_node_1(&self.container)?;
		}

		// Close
		let closure = Closure::once(move || crate::statics::remove_notification(rendered_at));

		if self.display_time > 0 {
			self.timeout = Some(window().set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), self.display_time)?);
		}

		self.listeners.push(closure);

		Ok(())
	}
}

impl Drop for Notification {
	fn drop(&mut self) {
		self.container.remove();

		if let Some(timeout) = self.timeout {
			window().clear_timeout_with_handle(timeout);
		}
	}
}