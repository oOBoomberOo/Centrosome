use super::{DataHolder, DataTree, FileType, Merger, Setup, ScriptType, MergeResult, GenerateResult};
use crate::utils::get_path_name;
use std::collections::HashMap;
use std::path::PathBuf;
use serde_json as js;

#[derive(Clone, Eq, PartialEq)]
pub struct Script {
	pub name: String,
	pub child: HashMap<String, Script>,
	data: Option<Vec<u8>>,
	script_type: ScriptType
}

impl Setup for Script {
	fn new(location: impl Into<PathBuf>, data: Option<Vec<u8>>, script_type: ScriptType) -> Script {
		let location = location.into();
		let name = get_path_name(&location);
		let child = HashMap::default();
		let data = data.map(Vec::from);
		Script { name, child, data, script_type }
	}
}

impl DataHolder for Script {
	fn data(&self) -> &Option<Vec<u8>> {
		&self.data
	}
}

impl DataTree<Script> for Script {
	fn create(name: impl Into<String>, child: HashMap<String, Script>, data: Option<Vec<u8>>, script_type: ScriptType) -> Script {
		let name = name.into();
		Script { name, child, data, script_type }
	}
}

use std::fmt;

impl fmt::Debug for Script {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.file_type() {
			FileType::File => write!(f, "```\n{}\n```", self.get_data()),
			FileType::Folder => write!(f, "{} {:#?}", self.name, self.child),
		}
	}
}

impl Merger for Script {
	fn merge(&self, other: Script, key: impl Into<String>) -> GenerateResult<Script> {
		match self.file_type() {
			FileType::File => {
				use super::data_structure::{Tag};

				match self.script_type {
					ScriptType::Tag => {
						let original: Tag = match self.data.clone() {
							Some(slice) => js::from_slice(&slice[..])?,
							None => Tag::default()
						};
						let mut prototype: Tag = match other.data.clone() {
							Some(slice) => js::from_slice(&slice[..])?,
							None => Tag::default()
						};

						let mut result = original;
						result.values.append(&mut prototype.values);

						let mut script = self.clone();
						script.data = js::to_vec_pretty(&result).ok();

						MergeResult::merge(script, 1, key)
					},
					_ => MergeResult::merge(other, 1, key)
				}
			},
			FileType::Folder => {
				let mut child = self.child.clone();
				let size = other.child
					.into_iter()
					.filter_map(|(key, script)| {
						match self.child.get(&key) {
							Some(original) => original.merge(script, &key).ok(),
							None => MergeResult::with_key(script, 1, &key)
						}
					})
					.map(|merge_result| {
						child.insert(merge_result.key, merge_result.script);
						merge_result.size
					})
					.sum();

				let name = other.name.to_owned();
				let data = None;
				let script_type = other.script_type;
				let script = Script { name, child, data, script_type };

				MergeResult::merge(script, size, key)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{Script, Setup, DataTree, HashMap, ScriptType};

	#[test]
	fn init_script() {
		let value = Script::new("bofuri/maple.syrup", None, ScriptType::Function);
		let expect = Script {
			name: "maple.syrup".to_string(),
			child: HashMap::default(),
			data: None,
			script_type: ScriptType::Function
		};

		assert_eq!(value, expect);
	}

	#[test]
	fn create_script() {
		let value = Script::create("Tanya von Degurechaff", HashMap::default(), None, ScriptType::Function);
		let expect = Script {
			name: "Tanya von Degurechaff".to_string(),
			child: HashMap::default(),
			data: None,
			script_type: ScriptType::Function
		};

		assert_eq!(value, expect);
	}
}