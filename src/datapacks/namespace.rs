use super::{
	CompiledResult, DataTree, GeneratedResult, MergedResult, Script, ScriptKind, TreeError,
};
use crate::utils::os_str_to_string;
use std::collections::HashSet;

/// Namespace represent a directory directly inside `/data` folder in a datapack
#[derive(Debug, Clone, Eq)]
pub struct Namespace {
	pub name: String,
	child: HashSet<Script>,
}

impl Namespace {
	fn new(name: impl Into<String>, child: HashSet<Script>) -> Namespace {
		let name = name.into();
		Namespace { name, child }
	}

	/// Inside namespace, folders will be split into "functions", "advancements", "tags" and etc.  
	/// This function will convert name of those folders into `ScriptKind`
	fn get_script_kind(name: &str) -> ScriptKind {
		match name {
			"tags" => ScriptKind::Tag,
			_ => ScriptKind::Generic,
		}
	}
}

use std::fs::{DirEntry, File};
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::ZipWriter;
impl DataTree for Namespace {
	fn generate(
		entry: DirEntry,
		_kind: ScriptKind,
		event: impl Fn(u64) + Copy,
	) -> GeneratedResult<Namespace> {
		if entry.metadata()?.is_dir() {
			let mut child: HashSet<Script> = HashSet::default();
			let mut size = 0;
			for entry in entry.path().read_dir()? {
				let entry: DirEntry = entry?;

				if entry.metadata()?.is_file() {
					match Script::generate(entry, ScriptKind::Generic, event) {
						Ok((script, child_size)) => {
							child.insert(script);
							size += child_size;
						}
						Err(error) => eprintln!("Unable to decode: {}", error),
					}
				} else {
					let name = os_str_to_string(&entry.file_name());
					let kind = Namespace::get_script_kind(&name);
					match Script::generate(entry, kind, event) {
						Ok((script, child_size)) => {
							child.insert(script);
							size += child_size;
						}
						Err(error) => eprintln!("Unable to decode: {}", error),
					}
				}
			}

			let name = os_str_to_string(&entry.file_name());
			let namespace = Namespace::new(name, child);
			Ok((namespace, size))
		} else {
			Err(TreeError::FileInNamespace(entry.path()))
		}
	}

	fn merge(&self, other: Namespace, event: impl Fn(u64) + Copy) -> MergedResult<Namespace> {
		let mut child = self.child.clone();
		for value in other.child {
			let script = match child.get(&value) {
				Some(original) => original.merge(value, event)?,
				None => value,
			};

			child.insert(script);
		}

		let name = other.name;
		let result = Namespace::new(name, child);

		Ok(result)
	}

	fn compile(
		&self,
		path: impl Into<PathBuf>,
		zip: &mut ZipWriter<File>,
		options: &FileOptions,
		event: impl Fn(u64) + Copy,
	) -> CompiledResult<()> {
		let path = path.into();
		zip.add_directory_from_path(&path, *options)?;

		for script in &self.child {
			let path = path.join(&script.name);
			script.compile(path, zip, options, event)?;
		}
		Ok(())
	}
}

impl PartialEq for Namespace {
	fn eq(&self, other: &Namespace) -> bool {
		self.name == other.name && self.child == other.child
	}
}

use std::hash::{Hash, Hasher};
impl Hash for Namespace {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.child.iter().for_each(|value| value.hash(state));
	}
}

impl From<DirEntry> for Namespace {
	fn from(entry: DirEntry) -> Namespace {
		let name = os_str_to_string(&entry.file_name());
		let child = HashSet::default();
		Namespace { name, child }
	}
}

#[cfg(test)]
mod tests {
	use super::super::FileType;
	use super::*;

	#[test]
	fn create_new_namespace() {
		assert_eq!(
			Namespace::new("Boomber", HashSet::default()),
			Namespace {
				name: "Boomber".to_string(),
				child: HashSet::default()
			}
		);
	}

	#[test]
	fn similar_namespaces() {
		let a = Namespace::new("Megumin", HashSet::default());
		let mut child = HashSet::default();
		child.insert(Script::new(
			"Chunchunmaru",
			HashSet::default(),
			ScriptKind::Generic,
			FileType::Directory,
		));
		let b = Namespace::new("Megumin", child);

		assert_ne!(a, b);
	}

	#[test]
	fn get_script_kind_tags() {
		assert_eq!(Namespace::get_script_kind("tags"), ScriptKind::Tag);
	}

	#[test]
	fn get_script_kind_generic() {
		assert_eq!(
			Namespace::get_script_kind("Ohayou Sekai~"),
			ScriptKind::Generic
		);
	}
}
