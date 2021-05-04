use diesel::SqliteConnection;
use diesel::prelude::*;
use rand::{Rng, thread_rng, distributions::Alphanumeric};

use crate::Result;
use super::QueryId;
use super::schema::*;
use super::models::{
	NewCanvasRef,
	CanvasModel, NewCanvasModel, EditCanvasModel,
	UserModel, NewUserModel
};

use circuit_sim_common::config::{StateInfo, ConfigJson, EditorType};

pub fn gen_unique_id(editor_type: EditorType) -> String {
	let prefix = match editor_type {
		EditorType::Canvas => "c",
		EditorType::CustomObject => "o"
	}.to_string();

	prefix + &thread_rng().sample_iter(Alphanumeric).take(8).map(char::from).collect::<String>()
}


// CANVAS REFS
pub fn delete_canvas_refs_by_c_id(id: i32, conn: &SqliteConnection) -> Result<()> {
	use self::canvas_refs::dsl::*;

	diesel::delete(canvas_refs.filter(c_id.eq(id))).execute(conn)?;

	Ok(())
}

pub fn create_canvas_refs(value: Vec<NewCanvasRef>, conn: &SqliteConnection) -> Result<()> {
	use self::canvas_refs::dsl::*;

	diesel::insert_or_ignore_into(canvas_refs).values(value).execute(conn)?;

	Ok(())
}


// CANVAS

pub fn create_canvas(item: NewCanvasModel, conn: &SqliteConnection) -> Result<String> {
	use self::canvi::dsl::*;

	let c_id = item.canvas_id.clone();

	diesel::insert_into(canvi).values(item).execute(conn)?;

	Ok(c_id)
}

pub fn edit_canvas(c_id: String, item: EditCanvasModel, conn: &SqliteConnection) -> Result<String> {
	use self::canvi::dsl::*;

	diesel::update(canvi.filter(canvas_id.eq(&c_id))).set(item).execute(conn)?;

	Ok(c_id)
}

pub fn delete_canvas(c_id: &str, conn: &SqliteConnection) -> Result<()> {
	use self::canvi::dsl::*;

	diesel::delete(canvi.filter(canvas_id.eq(c_id))).execute(conn)?;

	Ok(())
}

pub fn get_canvas_by_id(c_id: i32, conn: &SqliteConnection) -> Result<Option<CanvasModel>> {
	use self::canvi::dsl::*;
	Ok(canvi.filter(id.eq(c_id)).get_result(conn).optional()?)
}

pub fn get_canvas_by_canvas_id(c_id: String, conn: &SqliteConnection) -> Result<Option<CanvasModel>> {
	use self::canvi::dsl::*;
	Ok(canvi.filter(canvas_id.eq(c_id)).get_result(conn).optional()?)
}

pub fn get_canvas_by_user_id(u_id: QueryId, conn: &SqliteConnection) -> Result<Vec<CanvasModel>> {
	use self::canvi::dsl::*;
	Ok(canvi.filter(user_id.eq(u_id)).get_results(conn)?)
}

pub fn into_new_canvas_model(state: &StateInfo, json: ConfigJson) -> Result<NewCanvasModel> {
	Ok(NewCanvasModel {
		json: serde_json::to_string(&json)?,

		user_id: state.user_id,

		canvas_id: state.canvas_id.clone().unwrap(),
		revision: state.revision,

		forked_id: state.forked_id.clone(),
		title: state.title.clone(),
		description: state.description.clone(),

		private: state.private,

		created_at: state.created_at,
		updated_at: state.updated_at
	})
}

pub fn into_edit_canvas_model(state: &StateInfo, json: ConfigJson) -> Result<EditCanvasModel> {
	Ok(EditCanvasModel {
		json: Some(serde_json::to_string(&json)?),

		title: Some(state.title.clone()),
		description: Some(state.description.clone()),

		private: Some(state.private),

		updated_at: Some(state.updated_at)
	})
}

pub fn from_canvas_model(model: CanvasModel) -> Result<(StateInfo, ConfigJson)> {
	Ok((
		StateInfo {
			id: Some(model.id),

			user_id: model.user_id,

			private: model.private,

			canvas_id: Some(model.canvas_id),
			revision: model.revision,
			forked_id: model.forked_id,

			title: model.title,
			description: model.description,

			created_at: model.created_at,
			updated_at: model.updated_at,
			is_edited: false
		},
		serde_json::from_str(&model.json)?
	))
}

// USERS

pub fn create_user(item: &NewUserModel, conn: &SqliteConnection) -> Result<usize> {
	use self::users::dsl::*;
	Ok(diesel::insert_into(users).values(item).execute(conn)?)
}

pub fn get_user_by_id(u_id: QueryId, conn: &SqliteConnection) -> Result<Option<UserModel>> {
	use self::users::dsl::*;
	Ok(users.find(u_id).get_result(conn).optional()?)
}

pub fn get_user_by_username(u_name: &str, conn: &SqliteConnection) -> Result<Option<UserModel>> {
	use self::users::dsl::*;
	Ok(users.filter(name_lower.eq(u_name.to_lowercase())).get_result(conn).optional()?)
}