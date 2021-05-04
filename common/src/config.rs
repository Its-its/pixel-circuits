use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{CellPos, Dimensions, NodeDirection, NodeObjectSide, NodeValueTypes};
use crate::object::ObjectType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigJson {
	V1(CanvasStateJson)
}

impl ConfigJson {
	pub fn new(editor: CanvasStateJson) -> Self {
		ConfigJson::V1(editor)
	}

	pub fn into_inner_json(self) -> CanvasStateJson {
		match self {
			ConfigJson::V1(v) => v
		}
	}

	pub fn as_inner_json(&self) -> &CanvasStateJson {
		match self {
			ConfigJson::V1(v) => v
		}
	}

	pub fn as_mut_inner_json(&mut self) -> &mut CanvasStateJson {
		match self {
			ConfigJson::V1(v) => v
		}
	}

	pub fn editor_type(&self) -> EditorType {
		match self {
			ConfigJson::V1(_v) => EditorType::Canvas
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateInfo {
	/// ID of the Canvas/Object
	pub id: Option<i32>,

	/// ID of the creator.
	pub user_id: i32,

	/// Unique ID of the Canvas/Custom Object
	pub canvas_id: Option<String>,

	/// Revision number of the item.
	pub revision: i32,

	/// If the item was forked from another one
	pub forked_id: Option<String>,

	/// Title that the user gave the item.
	pub title: Option<String>,

	/// Description that the user gave the item.
	pub description: Option<String>,

	pub private: bool,

	pub created_at: i64,
	pub updated_at: i64,

	pub is_edited: bool
}

impl StateInfo {
	pub fn from_user_id(user_id: i32, date: i64) -> Self {
		StateInfo {
			id: None,

			user_id,

			created_at: date,
			updated_at: date,

			canvas_id: None,
			revision: 0,
			forked_id: None,

			title: None,
			description: None,

			private: false,
			is_edited: false
		}
	}
}


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum EditorType {
	Canvas = 0,
	CustomObject,
}

impl EditorType {
	pub fn num(&self) -> u8 {
		*self as u8
	}

	pub fn from_num(num: u8) -> Option<Self> {
		Some(match num {
			0 => Self::Canvas,
			1 => Self::CustomObject,
			_ => return None
		})
	}
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorJson {
	pub state: EditorStateJson
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorStateJson {
	Canvas(CanvasStateJson),
	CustomObject(Box<CustomObjectStateJson>)
}

impl EditorStateJson {
	pub fn as_canvas(&self) -> Option<&CanvasStateJson> {
		match self {
			Self::Canvas(c) => Some(c),
			Self::CustomObject(_) => None
		}
	}


	pub fn as_custom_object(&self) -> Option<&CustomObjectStateJson> {
		match self {
			Self::CustomObject(c) => Some(&*c),
			Self::Canvas(_) => None
		}
	}

	pub fn as_mut_canvas(&mut self) -> Option<&mut CanvasStateJson> {
		match self {
			Self::Canvas(c) => Some(c),
			Self::CustomObject(_) => None
		}
	}

	pub fn as_mut_custom_object(&mut self) -> Option<&mut CustomObjectStateJson> {
		match self {
			Self::CustomObject(c) => Some(&mut *c),
			Self::Canvas(_) => None
		}
	}

	pub fn into_custom_object(self) -> Option<CustomObjectStateJson> {
		match self {
			Self::CustomObject(c) => Some(*c),
			Self::Canvas(_) => None
		}
	}

	pub fn is_editor_canvas(&self) -> bool {
		matches!(self, EditorStateJson::Canvas(_))
	}

	pub fn is_editor_custom_object(&self) -> bool {
		matches!(self, EditorStateJson::CustomObject(_))
	}

	pub fn type_of(&self) -> EditorType {
		match self {
			Self::Canvas(_) => EditorType::Canvas,
			Self::CustomObject(_) => EditorType::CustomObject
		}
	}
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasStateJson {
	pub objects: Vec<ObjectsJson>,

	pub color_palette: Vec<(u16, u16, u16)>,

	/// palette index, Vec<CellPos>
	pub pixels: HashMap<usize, Vec<CellPos>>,

	pub text_objects: Vec<TextJson>
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomObjectStateJson {
	// CustomObject specific
	pub custom_object: ObjectsJson
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextJson {
	pub value: String,
	pub size: usize,

	pub pos_x: f64,
	pub pos_y: f64
}

// Object

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectsJson {
	Normal(ObjectNormalJson),
	Custom(Box<ObjectCustomJson>),
	Reference(ObjectReferenceJson)
}

impl ObjectsJson {
	pub fn as_custom(&self) -> Option<&ObjectCustomJson> {
		match self {
			ObjectsJson::Custom(c) => Some(&*c),
			_ => None
		}
	}

	pub fn as_mut_custom(&mut self) -> Option<&mut ObjectCustomJson> {
		match self {
			ObjectsJson::Custom(c) => Some(&mut *c),
			_ => None
		}
	}


	pub fn as_reference(&self) -> Option<&ObjectReferenceJson> {
		match self {
			ObjectsJson::Reference(v) => Some(v),
			_ => None
		}
	}

	pub fn into_reference(self) -> Option<ObjectReferenceJson> {
		match self {
			ObjectsJson::Reference(v) => Some(v),
			_ => None
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectNormalJson {
	pub type_of: ObjectType,

	pub id: usize,
	pub pos: CellPos,
	pub dim: Dimensions,
	pub nodes: Vec<NodeJson>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCustomJson {
	pub type_of: ObjectType,

	pub id: usize,
	pub pos: CellPos,
	pub dim: Dimensions,
	pub nodes: Vec<NodeJson>,

	pub inner_nodes: Vec<NodeJson>,
	pub node_positions: Vec<(NodeObjectSide, usize)>,
	pub object_name: Option<String>,

	pub canvas: CanvasStateJson
}

/// An Object which needs to be
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectReferenceJson {
	pub id: i32,
	pub canvas_id: String,

	pub type_of: ObjectType,

	pub pos: CellPos,
	pub nodes: Vec<NodeJson>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeJson {
	pub pos: CellPos,
	pub side: NodeObjectSide,
	pub direction: NodeDirection,
	pub accepts: NodeValueTypes,
	pub label: Option<String>,
	pub is_disabled: bool
}

impl NodeJson {
	pub fn is_input(&self) -> bool {
		matches!(self.direction, NodeDirection::Input)
	}

	pub fn is_output(&self) -> bool {
		!self.is_input()
	}
}