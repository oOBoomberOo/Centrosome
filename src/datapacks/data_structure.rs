use serde::{Deserialize, Serialize};

/// Representing JSON structure of "Tags" in datapack
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tag {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub replace: Option<bool>,
	pub values: Vec<String>,
}
