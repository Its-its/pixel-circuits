use std::mem;

use serde::{Deserialize, Serialize};

use crate::{CellPos, MARGIN_SIZE};

pub type DimSizeType = usize;
pub type DimSizeTuple = (DimSizeType, DimSizeType);


pub fn modify_usize_with_isize(value: usize, modifier: isize) -> Option<usize> {
	Some(if modifier.is_positive() {
		value + modifier as usize
	} else {
		value.checked_sub(modifier.abs() as usize)?
	})
}

pub fn offset_cell_pos(cell_pos: CellPos, offset: (isize, isize)) -> Option<CellPos> {
	Some((
		modify_usize_with_isize(cell_pos.0, offset.0)?,
		modify_usize_with_isize(cell_pos.1, offset.1)?
	))
}


#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NodeDirection {
	Input,
	Output
}

impl NodeDirection {
	pub fn is_input(&self) -> bool {
		matches!(self, Self::Input)
	}

	pub fn is_output(&self) -> bool {
		matches!(self, Self::Output)
	}
}


#[derive(Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
	Top,
	Right,
	Bottom,
	Left,
}


impl Side {
	pub fn is_top(self) -> bool {
		self == Self::Top
	}

	pub fn is_bottom(self) -> bool {
		self == Self::Bottom
	}

	pub fn is_left(self) -> bool {
		self == Self::Left
	}

	pub fn is_right(self) -> bool {
		self == Self::Right
	}

	pub fn opposite(self) -> Self {
		match self {
			Self::Top => Self::Bottom,
			Self::Left => Self::Right,
			Self::Right => Self::Left,
			Self::Bottom => Self::Top
		}
	}

	pub fn into_node_object_side(self, value: usize) -> NodeObjectSide {
		match self {
			Self::Top => NodeObjectSide::Top(value),
			Self::Left => NodeObjectSide::Left(value),
			Self::Right => NodeObjectSide::Right(value),
			Self::Bottom => NodeObjectSide::Bottom(value)
		}
	}

	pub fn as_facing_cell_pos(&self, cell_pos: CellPos) -> CellPos {
		match self {
			Self::Top => (cell_pos.0, cell_pos.1 - 1),
			Self::Left => (cell_pos.0 - 1, cell_pos.1),
			Self::Right => (cell_pos.0 + 1, cell_pos.1),
			Self::Bottom => (cell_pos.0, cell_pos.1 + 1)
		}
	}

	pub fn get_offset_pos<I: Default + std::ops::Neg<Output = I>>(self, value: I) -> (I, I) {
		match self {
			Self::Left => (-value, I::default()),
			Self::Right => (value, I::default()),
			Self::Top => (I::default(), -value),
			Self::Bottom => (I::default(), value)
		}
	}

	pub fn get_surrounding_cells(self, cell_pos: CellPos) -> [Option<CellPos>; 3] {
		match self {
			Self::Left => [
				offset_cell_pos(cell_pos, (0, -1)),
				offset_cell_pos(cell_pos, (-1, 0)),
				offset_cell_pos(cell_pos, (0, 1))
			],

			Self::Right => [
				offset_cell_pos(cell_pos, (0, -1)),
				offset_cell_pos(cell_pos, (1, 0)),
				offset_cell_pos(cell_pos, (0, 1))
			],

			Self::Top => [
				offset_cell_pos(cell_pos, (-1, 0)),
				offset_cell_pos(cell_pos, (0, -1)),
				offset_cell_pos(cell_pos, (1, 0))
			],

			Self::Bottom => [
				offset_cell_pos(cell_pos, (-1, 0)),
				offset_cell_pos(cell_pos, (0, 1)),
				offset_cell_pos(cell_pos, (1, 0))
			]
		}
	}

	pub fn get_surrounding_cells_with_side_its_on(self, cell_pos: CellPos) -> [(Option<CellPos>, Self); 3] {
		match self {
			Self::Left => [
				(offset_cell_pos(cell_pos, (0, -1)), Self::Top),
				(offset_cell_pos(cell_pos, (-1, 0)), Self::Left),
				(offset_cell_pos(cell_pos, (0, 1)), Self::Bottom)
			],

			Self::Right => [
				(offset_cell_pos(cell_pos, (0, -1)), Self::Top),
				(offset_cell_pos(cell_pos, (1, 0)), Self::Right),
				(offset_cell_pos(cell_pos, (0, 1)), Self::Bottom)
			],

			Self::Top => [
				(offset_cell_pos(cell_pos, (-1, 0)), Self::Left),
				(offset_cell_pos(cell_pos, (0, -1)), Self::Top),
				(offset_cell_pos(cell_pos, (1, 0)), Self::Right)
			],

			Self::Bottom => [
				(offset_cell_pos(cell_pos, (-1, 0)), Self::Left),
				(offset_cell_pos(cell_pos, (0, 1)), Self::Bottom),
				(offset_cell_pos(cell_pos, (1, 0)), Self::Right)
			]
		}
	}
}


impl PartialEq<Side> for NodeObjectSide {
	fn eq(&self, other: &Side) -> bool {
		&self.side() == other
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// The `NodeObjectSide` enum stores what nodes are connected to which sides.
pub enum NodeObjectSide {
	Left(usize),
	Top(usize),
	Right(usize),
	Bottom(usize)
}

impl NodeObjectSide {
	/// Returns the Cell Position based off of the Object Position and Dimensions.
	pub fn get_cell_pos(self, obj_dim: &Dimensions, obj_pos: CellPos) -> Option<CellPos> {
		let (pos_x, pos_y) = obj_pos;

		Some(match self {
			Self::Left(p) => {
				(
					pos_x.checked_sub(1)?,
					pos_y + obj_dim.height() - p - 1
				)
			}

			Self::Right(p) => {
				(
					pos_x + obj_dim.width(),
					pos_y + p
				)
			}

			Self::Top(p) => {
				(
					pos_x + p,
					pos_y.checked_sub(1)?
				)
			}

			Self::Bottom(p) => {
				(
					pos_x + obj_dim.width() - p - 1,
					pos_y + obj_dim.height()
				)
			}
		})
	}

	pub fn get_surrounding_cells(self, cell_pos: CellPos) -> [Option<CellPos>; 3] {
		self.side().get_surrounding_cells(cell_pos)
	}

	/// Returns the Cell Position in front of the current obj cell.
	// pub fn get_facing_cell_pos(&self, obj_dim: &Dimensions, obj_pos: CanvasPos) -> CellPos {
	// 	let cell_pos = self.get_cell_pos(obj_dim, obj_pos);
	// 	self.as_facing_cell_pos(cell_pos)
	// }

	pub fn as_facing_cell_pos(&self, cell_pos: CellPos) -> CellPos {
		self.side().as_facing_cell_pos(cell_pos)
	}

	pub fn as_facing_cell_pos_i32(&self, cell_pos: (i32, i32)) -> (i32, i32) {
		match self {
			Self::Left(_) => (cell_pos.0 - 1, cell_pos.1),
			Self::Right(_) => (cell_pos.0 + 1, cell_pos.1),
			Self::Top(_) => (cell_pos.0, cell_pos.1 - 1),
			Self::Bottom(_) => (cell_pos.0, cell_pos.1 + 1)
		}
	}

	/// Return new `NodeObjectSide` rotated clockwise.
	pub fn next_side_clockwise(self) -> Self {
		match self {
			Self::Left(i) => Self::Top(i),
			Self::Right(i) => Self::Bottom(i),
			Self::Top(i) => Self::Right(i),
			Self::Bottom(i) => Self::Left(i),
		}
	}

	/// Return new `NodeObjectSide` on opposite side.
	pub fn opposite_side(self) -> Self {
		match self {
			Self::Left(i) => Self::Right(i),
			Self::Right(i) => Self::Left(i),
			Self::Top(i) => Self::Bottom(i),
			Self::Bottom(i) => Self::Top(i),
		}
	}

	/// Returns `NodeObjectSide` side as `ObjectSide`
	pub fn side(&self) -> Side {
		match self {
			Self::Left(_) => Side::Left,
			Self::Top(_) => Side::Top,
			Self::Right(_) => Side::Right,
			Self::Bottom(_) => Side::Bottom
		}
	}

	pub fn is_top(self) -> bool {
		self == Side::Top
	}

	pub fn is_bottom(self) -> bool {
		self == Side::Bottom
	}

	pub fn is_left(self) -> bool {
		self == Side::Left
	}

	pub fn is_right(self) -> bool {
		self == Side::Right
	}

	/// Returns the inner value of the `NodeObjectSide`
	pub fn value(self) -> usize {
		match self {
			Self::Left(i) |
			Self::Right(i) |
			Self::Top(i) |
			Self::Bottom(i) => i
		}
	}

	pub fn previous(self) -> Self {
		let value = self.value();

		if value == 0 {
			self
		} else {
			self.at(value - 1)
		}
	}

	pub fn next(self) -> Self {
		self.at(self.value() + 1)
	}

	pub fn at(self, pos: usize) -> Self {
		match self {
			Self::Left(_) => Self::Left(pos),
			Self::Top(_) => Self::Top(pos),
			Self::Right(_) => Self::Right(pos),
			Self::Bottom(_) => Self::Bottom(pos),
		}
	}
}

impl From<(usize, usize)> for NodeObjectSide {
	fn from(i: (usize, usize)) -> Self {
		match i.0 {
			0 => Self::Left(i.1),
			1 => Self::Top(i.1),
			2 => Self::Right(i.1),
			3 => Self::Bottom(i.1),
			_ => panic!("Unknown size: {}", i.0)
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Dimensions(pub DimSizeType, pub DimSizeType);

impl Dimensions {
	/// Ensures width/height will never be 0.
	pub fn checked(width: DimSizeType, height: DimSizeType) -> Self {
		Self(std::cmp::max(width, 1), std::cmp::max(height, 1))
	}

	pub fn from_side(&self, side: Side) -> DimSizeType {
		match side {
			Side::Top |
			Side::Bottom => self.0,
			Side::Left |
			Side::Right => self.1
		}
	}

	pub fn rotate(&mut self) {
		mem::swap(&mut self.0, &mut self.1);
	}

	/// Increase X axis by 1.
	pub fn inc_width(&mut self) {
		self.set_width(self.width() + 1);
	}

	/// Increase Y axis by 1.
	pub fn inc_height(&mut self) {
		self.set_height(self.height() + 1);
	}

	/// Decrease X axis by 1. Stops at a size of 1.
	pub fn dec_width(&mut self) {
		let value = self.width();

		if value > 1 {
			self.set_width(value - 1);
		}
	}

	/// Decrease Y axis by 1. Stops at a size of 1.
	pub fn dec_height(&mut self) {
		let value = self.height();

		if value > 1 {
			self.set_height(value - 1);
		}
	}

	/// Returns length of X axis.
	pub fn width(&self) -> DimSizeType {
		self.0
	}

	/// Returns length of Y axis.
	pub fn height(&self) -> DimSizeType {
		self.1
	}

	/// Set length of X axis.
	pub fn set_width(&mut self, value: DimSizeType) {
		self.0 = value;
	}

	/// Set length of Y axis.
	pub fn set_height(&mut self, value: DimSizeType) {
		self.1 = value;
	}

	/// Returns the width of the cells.
	pub fn get_cell_width(&self, cell_size: f64) -> f64 {
		self.width() as f64 * cell_size
	}

	/// Returns the height of the cells.
	pub fn get_cell_height(&self, cell_size: f64) -> f64 {
		self.height() as f64 * cell_size
	}

	/// Gets the width subtracting the margins on both sides
	pub fn get_width_accountable(&self, cell_size: f64) -> f64 {
		self.get_cell_width(cell_size) - (MARGIN_SIZE * 2.0)
	}

	/// Gets the height subtracting the margins on both sides
	pub fn get_height_accountable(&self, cell_size: f64) -> f64 {
		self.get_cell_height(cell_size) - (MARGIN_SIZE * 2.0)
	}
}


pub struct Rectangle<I> {
	pub x: I,
	pub y: I,

	pub width: I,
	pub height: I
}

impl<I> Rectangle<I> {
	pub fn new(x: I, y: I, width: I, height: I) -> Self {
		Self { x, y, width, height }
	}
}

impl<I: Copy> Rectangle<I> {
	pub fn pos(&self) -> (I, I) {
		(self.x, self.y)
	}

	pub fn size(&self) -> (I, I) {
		(self.width, self.height)
	}
}

impl<I: Copy + Default> Rectangle<I> {
	pub fn zero_xy(&self) -> Rectangle<I> {
		Rectangle::new(I::default(), I::default(), self.width, self.height)
	}
}