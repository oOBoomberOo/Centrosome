use super::{
	CompiledResult, DataTree, FileType, GeneratedResult, MergedResult, ScriptKind, Tag, TreeError,
};
use serde::{Deserialize, Serialize};
use serde_json as js;
use serde_json::Result as JsResult;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;

/// Script is any files or directories that does not follow `Namespace` rule.
///
/// Script can also be a child of itself.
#[derive(Clone, Eq)]
pub struct Script {
	pub name: String,
	child: HashSet<Script>,
	kind: ScriptKind,
	file_type: FileType,
}

impl Script {
	pub fn new(
		name: impl Into<String>,
		child: HashSet<Script>,
		kind: ScriptKind,
		file_type: FileType,
	) -> Script {
		let name = name.into();
		Script {
			name,
			child,
			kind,
			file_type,
		}
	}

	/// Decode JSON data from slices
	fn decode<'a, T: Deserialize<'a>>(data: &'a [u8]) -> io::Result<T> {
		let result: T = js::from_slice(&data)?;
		Ok(result)
	}

	/// Encode JSON data to slices
	fn encode<T: Serialize>(data: &T) -> JsResult<Vec<u8>> {
		js::to_vec_pretty(data)
	}
}

use std::fs::{DirEntry, File};
use std::io::Write;
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::ZipWriter;
impl DataTree for Script {
	fn generate(
		entry: DirEntry,
		kind: ScriptKind,
		event: impl Fn(u64) + Copy,
	) -> GeneratedResult<Script> {
		if entry.metadata()?.is_file() {
			let data = fs::read(entry.path())?;
			let size = entry.metadata()?.len();
			let name = os_str_to_string(entry.file_name());
			let file_type = FileType::File(data);
			let script = Script::new(name, HashSet::default(), kind, file_type);
			event(size);
			Ok((script, size))
		} else {
			let mut child: HashSet<Script> = HashSet::default();
			let mut size = 0;
			for entry in entry.path().read_dir()? {
				let entry: DirEntry = entry?;
				match Script::generate(entry, kind, event) {
					Ok((script, child_size)) => {
						child.insert(script);
						size += child_size;
					}
					Err(error) => eprintln!("Unable to decode: {}", error),
				}
			}

			let name = os_str_to_string(&entry.file_name());
			let script = Script::new(name, child, kind, FileType::Directory);

			Ok((script, size))
		}
	}

	fn merge(&self, other: Script, event: impl Fn(u64) + Copy) -> MergedResult<Script> {
		match self.file_type.clone() {
			FileType::File(data) => {
				match self.kind {
					ScriptKind::Tag => {
						let original: Tag = match Script::decode(&data) {
							Ok(original) => original,
							Err(_) => return Ok(other),
						};
						let prototype: io::Result<Tag> = match other.file_type.clone() {
							FileType::File(data) => Script::decode(&data),
							FileType::Directory => {
								return Err(TreeError::MismatchType(
									self.name.clone(),
									other.name.clone(),
								))
							}
						};

						let mut prototype = match prototype {
							Ok(prototype) => prototype,
							Err(_) => return Ok(self.clone()),
						};

						let mut result = original;
						result.values.append(&mut prototype.values);
						let data = match Script::encode(&result) {
							Ok(x) => x,
							// Return `other` immediately if there are json error
							// Such as "Invalid syntax"
							Err(_error) => return Ok(other),
						};
						let size = data.len() as u64;

						let name = other.name;
						let child = other.child;
						let kind = other.kind;
						let file_type = FileType::File(data);

						event(size);
						let result = Script::new(name, child, kind, file_type);
						Ok(result)
					}
					ScriptKind::Generic => Ok(other),
					ScriptKind::None => Err(TreeError::UnknownFormat(self.name.clone())),
				}
			}
			FileType::Directory => {
				let mut child = self.child.clone();
				for value in other.child {
					let script = match child.get(&value) {
						Some(original) => original.merge(value, event),
						None => Ok(value),
					};

					match script {
						Ok(script) => {
							child.replace(script);
						}
						Err(error) => eprintln!("{}", error),
					};
				}
				let name = other.name;
				let kind = other.kind;
				let file_type = other.file_type;
				let result = Script::new(name, child, kind, file_type);
				Ok(result)
			}
		}
	}

	fn compile(
		&self,
		path: impl Into<PathBuf>,
		zip: &mut ZipWriter<File>,
		options: &FileOptions,
		event: impl Fn(u64) + Copy,
	) -> CompiledResult<()> {
		let path: PathBuf = path.into();
		
		match &self.file_type {
			FileType::Directory => {
				zip.add_directory_from_path(&path, *options)?;
				for script in &self.child {
					let child = path.join(&script.name);
					script.compile(child, zip, options, event)?;
				}
			}
			FileType::File(data) => {
				zip.start_file_from_path(&path, *options)?;
				zip.write_all(&data)?;
				event(data.len() as u64);
			}
		};

		Ok(())
	}
}

use crate::utils::os_str_to_string;
impl From<(DirEntry, ScriptKind)> for Script {
	fn from((entry, kind): (DirEntry, ScriptKind)) -> Script {
		let name = os_str_to_string(&entry.file_name());
		let child = HashSet::default();
		let file_type = {
			if entry.metadata().unwrap().is_file() {
				FileType::File(Vec::default())
			} else {
				FileType::Directory
			}
		};
		Script {
			name,
			child,
			kind,
			file_type,
		}
	}
}

impl PartialEq for Script {
	fn eq(&self, other: &Script) -> bool {
		self.name == other.name && self.kind == other.kind
	}
}

impl Hash for Script {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.kind.hash(state);
	}
}

use std::fmt;
impl fmt::Debug for Script {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.file_type {
			FileType::Directory => write!(f, "{:}: {:#?}", self.name, self.child),
			FileType::File(_) => write!(f, "{}", self.name),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn create_new_script_directory() {
		assert_eq!(
			Script::new(
				"hello_world",
				HashSet::default(),
				ScriptKind::Generic,
				FileType::Directory
			),
			Script {
				name: String::from("hello_world"),
				child: HashSet::default(),
				kind: ScriptKind::Generic,
				file_type: FileType::Directory
			}
		);
	}

	#[test]
	fn decode_jojo_name_json() {
		let data = r#"
		{
			"values": [
				"Jonathan Joestar",
				"Joseph Joestar",
				"Jotaro Kujo",
				"Josuke Higashikata",
				"Giorno Giovanna",
				"Jolyne Cujoh",
				"Johnny Joestar",
				"Josuke Higashikata"
			]
		}
		"#;

		let value: Tag = Script::decode(data.as_bytes()).unwrap();
		let expect = Tag {
			replace: None,
			values: vec![
				String::from("Jonathan Joestar"),
				String::from("Joseph Joestar"),
				String::from("Jotaro Kujo"),
				String::from("Josuke Higashikata"),
				String::from("Giorno Giovanna"),
				String::from("Jolyne Cujoh"),
				String::from("Johnny Joestar"),
				String::from("Josuke Higashikata"),
			],
		};

		assert_eq!(value, expect);
	}

	#[test]
	fn merge_jojo_and_fate_characters() {
		let jojo_data = r#"
		{
			"values": [
				"Jonathan Joestar",
				"Joseph Joestar",
				"Jotaro Kujo",
				"Josuke Higashikata",
				"Giorno Giovanna",
				"Jolyne Cujoh",
				"Johnny Joestar",
				"Josuke Higashikata"
			]
		}
		"#
		.as_bytes();
		let fate_data = r#"
		{
			"values": [
				"Shirou Emiya",
				"Saber",
				"Rin Tohsaka",
				"Archer",
				"Sakura Matou",
				"Sakura Matou",
				"Rider",
				"Illyasviel von Einzbern",
				"Kirei Kotomine",
				"Gilgamesh"
			]
		}
		"#
		.as_bytes();
		let expect_data = r#"
		{
			"values": [
				"Jonathan Joestar",
				"Joseph Joestar",
				"Jotaro Kujo",
				"Josuke Higashikata",
				"Giorno Giovanna",
				"Jolyne Cujoh",
				"Johnny Joestar",
				"Josuke Higashikata",
				"Shirou Emiya",
				"Saber",
				"Rin Tohsaka",
				"Archer",
				"Sakura Matou",
				"Sakura Matou",
				"Rider",
				"Illyasviel von Einzbern",
				"Kirei Kotomine",
				"Gilgamesh"
			]
		}
		"#
		.as_bytes();

		let jojo_script = Script::new(
			"jojo",
			HashSet::default(),
			ScriptKind::Tag,
			FileType::File(jojo_data.to_vec()),
		);
		let fate_script = Script::new(
			"fate",
			HashSet::default(),
			ScriptKind::Tag,
			FileType::File(fate_data.to_vec()),
		);

		let value = jojo_script.merge(fate_script, |_| {}).unwrap();
		let expect = Script::new(
			"fate",
			HashSet::default(),
			ScriptKind::Tag,
			FileType::File(expect_data.to_vec()),
		);

		assert_eq!(value, expect);
	}
}
