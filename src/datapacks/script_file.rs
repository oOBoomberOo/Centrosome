use std::path::PathBuf;
use super::{FileType, Script, DataHolder};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptFile {
	pub location: PathBuf,
	pub data: Vec<u8>,
	pub kind: FileType
}

impl ScriptFile {
	pub fn new(script: &Script, location: impl Into<PathBuf>) -> ScriptFile {
		let location = location.into();
		let data = match script.data().as_ref() {
			Some(v) => v.to_owned(),
			None => Vec::default()
		};
		let kind = script.file_type();

		ScriptFile { location, data, kind }
	}

	pub fn from_namespace(location: impl Into<PathBuf>) -> ScriptFile {
		let location = location.into();
		let data = Vec::default();
		let kind = FileType::Folder;

		ScriptFile { location, data, kind }
	}

	pub fn from_raw(location: impl Into<PathBuf>, data: Vec<u8>, kind: FileType) -> ScriptFile {
		let location = location.into();
		ScriptFile { location, data, kind }
	}
}