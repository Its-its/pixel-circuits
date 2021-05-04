use serde::{Serialize, Deserialize};
use diesel::{Queryable, Insertable};

use circuit_sim_common::http::UserInfo;

use crate::Result;
use super::{
	schema::*,
	QueryId
};


// CANVAS REFS

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "canvas_refs"]
#[primary_key(c_id, r_id)]
pub struct CanvasRef {
	pub c_id: QueryId,
	pub r_id: QueryId
}

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable)]
#[table_name = "canvas_refs"]
pub struct NewCanvasRef {
	pub c_id: QueryId,
	pub r_id: QueryId
}


// CANVAS

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Identifiable)]
#[table_name = "canvi"]
pub struct CanvasModel {
	pub id: QueryId,

	pub user_id: QueryId,

	pub canvas_id: String,
	pub revision: i32,

	pub forked_id: Option<String>,

	pub title: Option<String>,
	pub description: Option<String>,

	pub private: bool,

	pub json: String,

	pub created_at: i64,
	pub updated_at: i64
}

#[derive(Serialize, Deserialize, Debug, Insertable)]
#[table_name = "canvi"]
pub struct NewCanvasModel {
	pub user_id: QueryId,

	pub canvas_id: String,
	pub revision: i32,

	pub forked_id: Option<String>,

	pub title: Option<String>,
	pub description: Option<String>,

	pub private: bool,

	pub json: String,

	pub created_at: i64,
	pub updated_at: i64
}

#[derive(Serialize, Deserialize, Debug, AsChangeset)]
#[table_name = "canvi"]
pub struct EditCanvasModel {
	// pub user_id: QueryId,

	// pub canvas_id: String,
	// pub revision: i32,

	// pub forked_id: Option<String>,

	pub title: Option<Option<String>>,
	pub description: Option<Option<String>>,

	pub private: Option<bool>,

	pub json: Option<String>,

	// pub created_at: i64,
	pub updated_at: Option<i64>
}


// USERS

#[derive(Serialize, Deserialize, Debug, Default, Clone, Queryable, Identifiable)]
#[table_name = "users"]
pub struct UserModel {
	pub id: QueryId,

	pub name: String,
	pub name_lower: String,

	pub password: String,
	pub email: Option<String>,

	pub created_at: i64,
	pub updated_at: i64
}

impl Into<UserInfo> for UserModel {
	fn into(self) -> UserInfo {
		UserInfo {
			id: self.id,

			name: self.name,

			created_at: self.created_at,
			updated_at: self.updated_at
		}
	}
}


#[derive(Serialize, Deserialize, Debug, Clone, Insertable)]
#[table_name = "users"]
pub struct NewUserModel {
	pub name: String,
	pub name_lower: String,

	pub password: String,
	pub email: Option<String>,

	pub created_at: i64,
	pub updated_at: i64
}

impl NewUserModel {
	pub fn new_with_name_and_password(username: String, password: String) -> Result<Self> {
		Ok(NewUserModel {
			name_lower: username.to_lowercase(),
			name: username,

			password: crate::hash_password(password)?,
			email: None,

			created_at: crate::now(),
			updated_at: crate::now()
		})
	}
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, AsChangeset)]
#[table_name = "users"]
pub struct EditUserModel {
	pub name: Option<String>,
	pub name_lower: Option<String>,

	pub password: Option<String>,
	pub email: Option<Option<String>>,

	pub updated_at: Option<i64>
}