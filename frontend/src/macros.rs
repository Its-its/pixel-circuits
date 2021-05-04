#[macro_export]
macro_rules! log {
	( $( $t:tt )* ) => {
		// Rust Analyzer warning fix.
		#[allow(unused_unsafe, unsafe_code)]
		unsafe { web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!( $( $t )* ))); };
	}
}


// https://stackoverflow.com/a/30275713

#[macro_export]
macro_rules! register_tickable {
	($ty:ty) => {
		use crate::objects::{Tickable, ImplTickable};

		impl ImplTickable for $ty {
			fn as_tickable_ref(&self) -> Option<&dyn Tickable> {
				Some(self)
			}

			fn as_tickable_mut(&mut self) -> Option<&mut dyn Tickable> {
				Some(self)
			}
		}
	};

	(!$ty:ty) => {
		use crate::objects::{Tickable, ImplTickable};

		impl ImplTickable for $ty {
			fn as_tickable_ref(&self) -> Option<&dyn Tickable> {
				None
			}

			fn as_tickable_mut(&mut self) -> Option<&mut dyn Tickable> {
				None
			}
		}
	};
}