use std::env;
use std::io;

#[macro_use] extern crate diesel;

use diesel::SqliteConnection;
use pwhash::bcrypt;

use actix_web::{
	middleware::Logger,
	get, post, delete,
	web::{self, HttpResponse},
	App, HttpServer, Result as WebResult
};

use actix_session::{Session, CookieSession};

use circuit_sim_common::config::StateInfo;
use circuit_sim_common::http::{
	CanvasJsonReqResp,
	LoginForm,
	LoadResponse,
	SaveResponse,
	RegisterResponse, LoginResponse, AuthedResponse,
	ListCanviResponse
};

use database::{
	Connection, QueryId,
	models::{
		NewUserModel,
		NewCanvasRef
	},
	objects::{
		gen_unique_id,

		create_canvas_refs,

		delete_canvas,
		edit_canvas,
		create_canvas,

		into_new_canvas_model,
		into_edit_canvas_model,
		from_canvas_model,

		get_canvas_by_canvas_id,
		get_canvas_by_user_id,

		get_user_by_id,
		get_user_by_username,
		create_user
	}
};

mod error;
pub mod database;

pub use error::{Error, Result, AuthError};

#[get("/")]
async fn index() -> WebResult<actix_files::NamedFile> {
	Ok(actix_files::NamedFile::open("../frontend/static/index.html")?)
}

// Edit or create a canvas
#[get("/edit")]
async fn create() -> WebResult<actix_files::NamedFile> {
	Ok(actix_files::NamedFile::open("../frontend/static/index.html")?)
}

#[get("/edit/{id}")]
async fn edit() -> WebResult<actix_files::NamedFile> {
	Ok(actix_files::NamedFile::open("../frontend/static/index.html")?)
}

// View an already-made canvas
#[get("/view/{id}")]
async fn view() -> WebResult<actix_files::NamedFile> {
	Ok(actix_files::NamedFile::open("../frontend/static/index.html")?)
}

#[get("/authed")]
async fn authed(session: Session, pool: web::Data<Connection>) -> WebResult<HttpResponse> {
	let u_id = session.get::<QueryId>("id")?.ok_or_else::<actix_web::Error, _>(|| AuthError::Session.into())?;

	let conn = pool.connection()?;

	let user = web::block(move || get_user_by_id(u_id, &conn))
		.await?.ok_or_else::<actix_web::Error, _>(|| AuthError::UnableToFindUser.into())?;

	Ok(HttpResponse::Ok().json(AuthedResponse::Ok(user.into())))
}


#[post("/login")]
async fn login(session: Session, form: web::Json<LoginForm>, pool: web::Data<Connection>) -> WebResult<HttpResponse> {
	let inner = form.into_inner();
	let (username, password) = (inner.username, inner.password);

	let conn = pool.connection()?;

	let user = web::block(move || get_user_by_username(&username, &conn))
		.await?.ok_or_else::<actix_web::Error, _>(|| AuthError::UnableToFindUser.into())?;

	let can_login = bcrypt::verify(password.as_str(), user.password.as_str());

	if can_login {
		session.set("id", user.id).map_err::<actix_web::Error, _>(|_| AuthError::Session.into())?;
	}

	Ok(HttpResponse::Ok().json(LoginResponse::Ok(can_login)))
}

#[post("/register")]
async fn register(session: Session, form: web::Json<LoginForm>, pool: web::Data<Connection>) -> WebResult<HttpResponse> {
	let inner = form.into_inner();
	let (username, password) = (inner.username, inner.password);

	let conn = pool.connection()?;

	let user = web::block(move || -> Result<_> {
		let user_exists = get_user_by_username(&username, &conn)?.is_some();

		if user_exists {
			Ok(None)
		} else {
			create_user(&NewUserModel::new_with_name_and_password(username.clone(), password)?, &conn)?;

			// Get the user to return the id of it.
			get_user_by_username(&username, &conn)
		}

	})
	.await?.ok_or_else::<actix_web::Error, _>(|| AuthError::UserAlreadyExists.into())?;

	session.set("id", user.id).map_err::<actix_web::Error, _>(|_| AuthError::Session.into())?;

	Ok(HttpResponse::Ok().json(RegisterResponse::Ok(user.into())))
}



#[delete("/canvas/{id}")]
async fn delete_canvas_http(id: web::Path<String>, session: Session, pool: web::Data<Connection>) -> WebResult<HttpResponse> {
	let c_id = id.into_inner();

	let conn = pool.connection()?;

	let u_id = session.get::<QueryId>("id")?.ok_or_else::<actix_web::Error, _>(|| AuthError::Session.into())?;

	let removed = web::block(move || -> Result<bool> {
		let canvas = load_canvas_by_c_id(c_id.clone(), &conn)?;

		let same_user = canvas.map(|c| c.info.user_id == u_id).unwrap_or_default();

		if same_user {
			delete_canvas(&c_id, &conn)?;
		}

		Ok(same_user)
	})
	.await?;

	if removed {
		Ok(HttpResponse::Ok().finish())
	} else {
		Ok(HttpResponse::Unauthorized().finish())
	}
}

#[get("/canvas/{id}")]
async fn get_canvas_http(id: web::Path<String>, pool: web::Data<Connection>) -> WebResult<HttpResponse> {
	let conn = pool.connection()?;

	let resp = web::block(move || load_canvas_by_c_id(id.into_inner(), &conn))
	.await?.ok_or_else::<actix_web::Error, _>(|| AuthError::UnableToFindCanvas.into())?;

	Ok(HttpResponse::Ok().json(LoadResponse::Ok(resp)))
}


#[get("/list/canvi")]
async fn list_canvi(session: Session, pool: web::Data<Connection>) -> WebResult<HttpResponse> {
	let conn = pool.connection()?;

	let u_id = session.get::<QueryId>("id")?.ok_or_else::<actix_web::Error, _>(|| AuthError::Session.into())?;

	let canvi = web::block(move || -> Result<Vec<CanvasJsonReqResp>> {
		let mut canvi = Vec::new();

		for canvas in get_canvas_by_user_id(u_id, &conn)? {
			let (info, json) = from_canvas_model(canvas)?;

			canvi.push(CanvasJsonReqResp {
				info,
				json
			});
		}

		Ok(canvi)
	})
	.await?;

	Ok(HttpResponse::Ok().json(ListCanviResponse::Ok(canvi)))
}


fn load_canvas_by_c_id(c_id: String, conn: &SqliteConnection) -> Result<Option<CanvasJsonReqResp>> {
	if let Some(canvas) = get_canvas_by_canvas_id(c_id, conn)? {
		let (info, json) = from_canvas_model(canvas)?;

		Ok(Some(CanvasJsonReqResp {
			info,
			json
		}))
	} else {
		Ok(None)
	}
}

#[post("/save")]
async fn save(session: Session, form: web::Json<CanvasJsonReqResp>, pool: web::Data<Connection>) -> WebResult<HttpResponse> {
	let inner = form.into_inner();
	let conn = pool.connection()?;

	let u_id = session.get::<QueryId>("id")?.ok_or_else::<actix_web::Error, _>(|| AuthError::NotAuthenticated.into())?;

	let authed_user = web::block(move || get_user_by_id(u_id, &conn))
		.await?.ok_or_else::<actix_web::Error, _>(|| AuthError::UnableToFindUser.into())?;

	// Ensure Web User and Canvas Creator are the same.
	if inner.info.user_id != authed_user.id {
		return Err(AuthError::UnexpectedUser.into());
	}

	let conn = pool.connection()?;

	let resp = web::block(move || -> Result<StateInfo> {
		let mut state = inner.info;

		let json = inner.json.as_inner_json();

		{
			let refs: Vec<NewCanvasRef> = json.objects.iter()
				.filter_map(|j| j.as_reference().map(|r| NewCanvasRef { r_id: r.id, c_id: state.id.unwrap() }))
				.collect();

			println!("REFERENCES: {:?}", refs);

			create_canvas_refs(refs, &conn)?;
		}

		if state.canvas_id.is_some() {
			let c_id = state.canvas_id.clone().unwrap();
			state.updated_at = now();

			println!("Updating Canvas: {}", c_id);

			edit_canvas(c_id, into_edit_canvas_model(&state, inner.json)?, &conn)?;

			Ok(state)
		} else {
			state.canvas_id = Some(gen_unique_id(inner.json.editor_type()));

			// TODO: Ensure ID not already taken.

			println!("Creating Canvas: {:?}", state.canvas_id);

			create_canvas(into_new_canvas_model(&state, inner.json)?, &conn)?;

			Ok(state)
		}
	}).await?;

	Ok(HttpResponse::Ok().json(SaveResponse::Ok(resp)))
}


#[actix_rt::main]
async fn main() -> io::Result<()> {
	let conn = Connection::new();
	conn.init_sql().expect("DATABASE SQL INIT ERROR");

	env::set_var("RUST_LOG", "actix_web=info");
	env_logger::init();

	HttpServer::new(move || {
		App::new()
		.data(conn.clone())
		.wrap(
			CookieSession::private("132lkfg5$l2h6@vlegfk^sagf*23490u".as_bytes())
			.name("auth")
			.max_age(60 * 60 * 24 * 365)
			.secure(false)
		)
		.wrap(Logger::new(r##"%a "%r" %s %b %D"##))
		.service(index)
		.service(create)
		.service(edit)
		.service(view)

		.service(list_canvi)
		.service(delete_canvas_http)

		.service(get_canvas_http)
		.service(save)

		.service(login)
		.service(register)
		.service(authed)

		.service(actix_files::Files::new("/static", "../frontend/public").show_files_listing())
		.service(actix_files::Files::new("/", "../frontend/dist").show_files_listing())
	})
	.bind("0.0.0.0:8080")?
	.run()
	.await
}

pub fn now() -> i64 {
	chrono::Utc::now().timestamp_millis()
}

pub fn hash_password(password: String) -> Result<String> {
	Ok(bcrypt::hash(password)?)
}