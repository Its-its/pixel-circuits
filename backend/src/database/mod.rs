use diesel::Connection as _Connection;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use r2d2::PooledConnection;

pub mod models;
pub mod schema;
pub mod objects;

use crate::{Result, AuthError};

pub type QueryId = i32;
pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;


#[derive(Clone)]
pub struct Connection(DbPool);


impl Connection {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		let database_url = "../app/circuitsim.db";

		let manager = ConnectionManager::<SqliteConnection>::new(database_url);

		let pool = r2d2::Pool::builder()
			.build(manager)
			.expect("Failed to create pool.");

		Self(pool)
	}

	pub fn init_sql(&self) -> Result<()> {
		let conn = self.connection()?;

		// Users

		conn.execute(
			"CREATE TABLE IF NOT EXISTS users (
				id          INTEGER PRIMARY KEY,

				name        TEXT,

				created_at  LONG NOT NULL,
				updated_at  LONG NOT NULL
			)"
		)?;

		conn.execute(
			"CREATE UNIQUE INDEX IF NOT EXISTS users_name on users ( name )"
		)?;


		// CANVAS

		conn.execute( // RIGHT?! PLURALITY OF CANVAS?!
			"CREATE TABLE IF NOT EXISTS canvi (
				id                 INTEGER PRIMARY KEY,

				user_id            INTEGER NOT NULL,

				canvas_id          TEXT NOT NULL,
				revision           INTEGER NOT NULL,

				forked_id          TEXT,
				title              TEXT,
				description        TEXT,

				private            BOOL NOT NULL DEFAULT true,

				json               TEXT,

				created_at         LONG NOT NULL,
				updated_at         LONG NOT NULL
			)"
		)?;

		conn.execute(
			"CREATE UNIQUE INDEX IF NOT EXISTS canvi_canvas_ids on canvi ( canvas_id )"
		)?;

		// Used for checking if any other canvas's rely on Custom Object.
		// If no other Canvas's rely on Custom Object; delete.
		conn.execute(
			"CREATE TABLE IF NOT EXISTS canvas_refs (
				c_id   INTEGER,
				r_id   INTEGER,

				PRIMARY KEY(c_id,r_id)
			)"
		)?;

		Ok(())
	}

	pub fn connection(&self) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>> {
		self.0.get().map_err(|e| AuthError::Database(e).into())
	}
}