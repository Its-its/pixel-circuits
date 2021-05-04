use serde::{Deserialize, Serialize};

use crate::config::{ConfigJson, StateInfo};

pub type Response<T> = Result<T, String>;

// HREF /save
pub type SaveResponse = Response<StateInfo>;

// HREF /load
pub type LoadResponse = Response<CanvasJsonReqResp>;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadRequest {
	pub id: String
}

// HREF /register
pub type RegisterResponse = Response<UserInfo>;

// HREF /login
pub type LoginResponse = Response<bool>;

// HREF /authed
pub type AuthedResponse = Response<UserInfo>;

// HREF /list/canvi
pub type ListCanviResponse = Response<Vec<CanvasJsonReqResp>>;

// Save Request / Load Response


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasJsonReqResp {
	pub info: StateInfo,
	pub json: ConfigJson
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LoginForm {
	pub username: String,
	pub password: String
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UserInfo {
	pub id: i32,

	pub name: String,

	pub created_at: i64,
	pub updated_at: i64
}