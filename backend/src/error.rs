use std::fmt::{self, Display, Formatter};

use actix_web::{
	dev::HttpResponseBuilder,
	http::StatusCode,
	error::ResponseError,
	HttpResponse
};

use actix_web::Error as ActixError;
use diesel::result::Error as QueryError;
use diesel::r2d2::PoolError;
use serde_json::Error as JsonError;
use pwhash::error::Error as PwHashError;

use circuit_sim_common::http::Response;


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	PwHashError(PwHashError),
	// ActixError(ActixError),
	QueryError(QueryError),
	JsonError(JsonError),

	Auth(AuthError)
}

#[derive(Debug)]
pub enum AuthError {
	Database(PoolError),

	NotAuthenticated,

	UserAlreadyExists,
	UnableToFindUser,
	UnexpectedUser,

	UnableToFindCanvas,
	UnableToConvertAReference,

	LoginFailed,
	Session
}

impl ResponseError for Error {
	fn error_response(&self) -> HttpResponse {
		HttpResponseBuilder::new(self.status_code())
			.json(Response::<String>::Err(self.to_string()))
	}

	fn status_code(&self) -> StatusCode {
		StatusCode::INTERNAL_SERVER_ERROR
	}
}


impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Error::PwHashError(e) => e.fmt(f),
			Error::QueryError(e) => e.fmt(f),
			Error::JsonError(e) => e.fmt(f),
			Error::Auth(e) => f.write_str(match e {
				AuthError::UnableToConvertAReference => "Unable to convert an Custom Object Reference to itself",
				AuthError::UserAlreadyExists => "User already exists",
				AuthError::UnableToFindCanvas => "Unable to find canvas",
				AuthError::NotAuthenticated => "User Not Authenticated",
				AuthError::UnableToFindUser => "Unable to find user",
				AuthError::UnexpectedUser => "Unexpected User",
				AuthError::LoginFailed => "Failed to Login",
				AuthError::Session => "Session Error",
				AuthError::Database(e) => return e.fmt(f)
			})
		}
	}
}

impl From<AuthError> for Error {
	fn from(error: AuthError) -> Self {
		Error::Auth(error)
	}
}

impl From<QueryError> for Error {
	fn from(error: QueryError) -> Self {
		Error::QueryError(error)
	}
}

impl From<JsonError> for Error {
	fn from(error: JsonError) -> Self {
		Error::JsonError(error)
	}
}

impl From<PwHashError> for Error {
	fn from(error: PwHashError) -> Self {
		Error::PwHashError(error)
	}
}

impl Into<ActixError> for AuthError {
	fn into(self) -> ActixError {
		HttpResponse::InternalServerError()
		.json(Response::<String>::Err(format!("{:?}", self))).into()
	}
}