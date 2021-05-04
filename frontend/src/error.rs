use serde::Serialize;
use serde_json::Error as JsonError;
use wasm_bindgen::JsValue;

use std::sync::PoisonError;

pub type Result<T> = std::result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
	// External
	JsValue(JsValue),

	Json(JsonError),

	// Core
	Config(ConfigError),
	Display(DisplayError),
	Editing(EditingError),

	// Backend
	String(String)
}


#[derive(Debug, Serialize)]
pub enum ConfigError {
	InvalidSaveState,
	UnexpectedObject,
	UnexpectedNodeType,
	UnexpectedConnection,
	LoadingError(u16, String)
}

#[derive(Debug, Serialize)]
pub enum DisplayError {
	InvalidState,
	ParentDoesntExist
}

#[derive(Debug, Serialize)]
pub enum EditingError {
	UnableToFindObject,

	ValueIsNotGpio,
	ValueIsNotCurrent,

	LineEnumIsNotLineNode,
	LineEnumIsNotLine,

	PoisonError,

	UnableToUpgradeNodeDropped,
	UnableToUpgradeConnectionDropped,

	LineTypeIsNone,

	NodeIsNotInput
}


impl From<String> for Error {
	fn from(value: String) -> Self {
		Error::String(value)
	}
}

impl<A> From<PoisonError<A>> for Error {
	fn from(_: PoisonError<A>) -> Self {
		Error::Editing(EditingError::PoisonError)
	}
}

impl From<JsValue> for Error {
	fn from(value: JsValue) -> Self {
		Error::JsValue(value)
	}
}

impl From<JsonError> for Error {
	fn from(value: JsonError) -> Self {
		Error::Json(value)
	}
}


impl Into<Error> for ConfigError {
	fn into(self) -> Error {
		Error::Config(self)
	}
}

impl Into<Error> for DisplayError {
	fn into(self) -> Error {
		Error::Display(self)
	}
}

impl Into<Error> for EditingError {
	fn into(self) -> Error {
		Error::Editing(self)
	}
}

// Used for lib.rs fn main Result return.
impl Into<JsValue> for Error {
	fn into(self) -> JsValue {
		match self {
			Error::JsValue(value) => value,
			Error::String(e) => JsValue::from_str(&e),
			Error::Json(e) => JsValue::from_str(&e.to_string()),
			Error::Config(e) => JsValue::from_serde(&e).unwrap(),
			Error::Display(e) => JsValue::from_serde(&e).unwrap(),
			Error::Editing(e) => JsValue::from_serde(&e).unwrap()
		}
	}
}