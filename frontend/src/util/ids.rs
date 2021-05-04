use std::sync::atomic::{AtomicUsize, Ordering};

// Each Id Increments by 1 whenever it's defined.
// Allows for EASY PartialEq

// No Zeros. Zero is a default.
pub static LAST_OBJECT_ID: AtomicUsize = AtomicUsize::new(1);
pub static LAST_TEXT_ID: AtomicUsize = AtomicUsize::new(1);


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectId(pub usize);


macro_rules! gen_impl {
	($type:ident, $cache:ident) => {
		impl $type {
			pub fn gen_id() -> Self {
				Self($cache.fetch_add(1, Ordering::SeqCst))
			}

			pub fn reset() {
				$cache.store(1, Ordering::Relaxed);
			}

			pub fn empty() -> Self {
				Self(0)
			}
		}

		impl Into<usize> for $type {
			fn into(self) -> usize {
				self.0
			}
		}

		impl PartialEq<usize> for $type {
			fn eq(&self, other: &usize) -> bool {
				self.0 == *other
			}
		}
	}
}


gen_impl!(ObjectId, LAST_OBJECT_ID);
gen_impl!(TextId, LAST_TEXT_ID);