table! {
	users(id) {
		id -> Integer,

		name -> Text,
		name_lower -> Text,
		password -> Text,
		email -> Nullable<Text>,

		created_at -> BigInt,
		updated_at -> BigInt,
	}
}

table! {
	canvi(id) {
		id -> Integer,

		user_id -> Integer,

		canvas_id -> Text,
		revision -> Integer,

		forked_id -> Nullable<Text>,
		title -> Nullable<Text>,
		description -> Nullable<Text>,

		private -> Bool,

		json -> Text,

		created_at -> BigInt,
		updated_at -> BigInt,
	}
}

table! {
	canvas_refs(c_id, r_id) {
		c_id -> Integer,
		r_id -> Integer,
	}
}