pub type CellPos = (usize, usize);
pub type CanvasPos = (f64, f64);

pub mod config;
pub mod object;
pub mod size;
pub mod http;

pub use object::{NodeValueTypes, MARGIN_SIZE};
pub use size::*;