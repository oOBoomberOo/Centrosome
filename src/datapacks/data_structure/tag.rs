use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Tag {
	#[serde(skip_serializing_if="Option::is_none")]
	pub replace: Option<bool>,
	pub values: Vec<String>
}