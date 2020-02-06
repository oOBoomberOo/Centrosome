use super::{DataHolder, DataTree, FileType, Merger, Setup, EncodingFormat};
use crate::utils::get_path_name;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Eq, PartialEq)]
pub struct Script {
	pub name: String,
	pub child: HashMap<String, Script>,
	data: Option<Vec<u8>>,
	format: EncodingFormat
}

impl Setup for Script {
	fn new(location: impl Into<PathBuf>, data: Option<Vec<u8>>) -> Script {
		let location = location.into();
		let name = get_path_name(&location);
		let child = HashMap::default();
		let data = data.map(Vec::from);
		let format = EncodingFormat::Utf8;
		Script { name, child, data, format }
	}
}

impl DataHolder for Script {
	fn data(&self) -> &Option<Vec<u8>> {
		&self.data
	}
}

impl DataTree<Script> for Script {
	fn create(name: String, child: HashMap<String, Script>, data: Option<Vec<u8>>) -> Script {
		Script { name, child, data, format: EncodingFormat::Utf8 }
	}
}

use std::fmt;

impl fmt::Debug for Script {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.file_type() {
			FileType::File => write!(f, "{}", self.name),
			FileType::Folder => write!(f, "{} {:#?}", self.name, self.child),
		}
	}
}

impl Merger for Script {
	fn merge(&self, other: Script) -> (Script, u64) {
		match self.file_type() {
			FileType::File => (other, 1),
			FileType::Folder => {
				let mut result_child = self.child.clone();
				let mut counts = 0;
				for (key, script) in other.child {
					let (script, count) = match self.child.get(&key) {
						Some(original) => original.merge(script),
						None => (script, 1),
					};

					counts += count;
					result_child.insert(key, script);
				}

				let name = other.name.to_owned();
				let child = result_child;
				let data = None;
				let format = EncodingFormat::Utf8;

				(Script { name, child, data, format }, counts)
			}
		}
	}
}