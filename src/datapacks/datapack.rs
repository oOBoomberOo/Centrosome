use super::{
	CompiledResult, DataTree, GeneratedResult, MergedResult, Namespace, Script, ScriptKind,
	TreeError,
};
use crate::utils::os_str_to_string;
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::ZipWriter;

/// A struct representing a datapack as a whole
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Datapack {
	location: PathBuf,
	pub name: String,
	child: HashSet<Namespace>,
	files: HashSet<Script>,
}

impl Datapack {
	fn new(
		location: impl Into<PathBuf>,
		name: impl Into<String>,
		child: HashSet<Namespace>,
		files: HashSet<Script>,
	) -> Datapack {
		let location = location.into();
		let name = name.into();
		Datapack {
			location,
			name,
			child,
			files,
		}
	}

	/// Because `Datapack` doesn't have the same data structure as the one implementing `DataTree`.
	/// It cannot implement that trait itself so this function mimick `DataTree`'s generate() function
	pub fn generate(
		path: impl Into<PathBuf>,
		event: impl Fn(u64) + Copy,
	) -> GeneratedResult<Datapack> {
		let path = path.into();
		if path.is_dir() {
			let mut child = HashSet::default();
			let mut files = HashSet::default();
			let mut size = 0;
			for entry in path.read_dir()? {
				let entry = entry?;
				let name = os_str_to_string(entry.file_name());

				if name != "data" {
					match Script::generate(entry, ScriptKind::Generic, event) {
						Ok((script, child_size)) => {
							files.insert(script);
							size += child_size;
						}
						Err(error) => eprintln!("{}", error),
					}
				}
			}

			for entry in path.join("data").read_dir()? {
				let entry = entry?;

				match Namespace::generate(entry, ScriptKind::default(), event) {
					Ok((namespace, child_size)) => {
						child.insert(namespace);
						size += child_size;
					}
					Err(error) => match error {
						TreeError::FileInNamespace(_) => (),
						_ => eprintln!("{}", error),
					},
				}
			}

			let name = os_str_to_string(&path.as_os_str());
			let location = path;
			let datapack = Datapack::new(location, name, child, files);
			Ok((datapack, size))
		} else {
			Err(TreeError::FileInDatapack(path))
		}
	}

	/// Because `Datapack` doesn't have the same data structure as the one implementing `DataTree`.
	/// It cannot implement that trait itself so this function mimick `DataTree`'s merge() function
	pub fn merge(&self, other: Datapack, event: impl Fn(u64) + Copy) -> MergedResult<Datapack> {
		let mut child = self.child.clone();
		for value in other.child {
			let namespace = match child.get(&value) {
				Some(original) => original.merge(value, event)?,
				None => value,
			};

			child.replace(namespace);
		}

		let mut files = self.files.clone();
		for value in other.files {
			let script = match files.get(&value) {
				Some(original) => original.merge(value, event)?,
				None => value,
			};

			files.replace(script);
		}

		let result = Datapack::new(&self.location, &self.name, child, files);
		Ok(result)
	}

	/// Because `Datapack` doesn't have the same data structure as the one implementing `DataTree`.
	/// It cannot implement that trait itself so this function mimick `DataTree`'s compile() function
	pub fn compile(
		&self,
		output_location: impl Into<PathBuf>,
		options: &FileOptions,
		event: impl Fn(u64) + Copy,
	) -> CompiledResult<()> {
		let output_location = output_location.into();
		let writer = File::create(&output_location)?;
		let mut zip = ZipWriter::new(writer);
		let local_path = PathBuf::default();

		for namespace in &self.child {
			let path = local_path.join("data").join(&namespace.name);
			namespace.compile(path, &mut zip, options, event)?;
		}

		for file in &self.files {
			let path = local_path.join(&file.name);
			file.compile(path, &mut zip, options, event)?;
		}

		zip.finish()?;

		Ok(())
	}
}

use std::fs::DirEntry;
impl From<DirEntry> for Datapack {
	fn from(entry: DirEntry) -> Datapack {
		let location = entry.path();
		let name = os_str_to_string(&entry.file_name());
		let child = HashSet::default();
		let files = HashSet::default();
		Datapack {
			location,
			name,
			child,
			files,
		}
	}
}

impl From<PathBuf> for Datapack {
	fn from(path: PathBuf) -> Datapack {
		let name = os_str_to_string(path.file_name().unwrap());
		let child = HashSet::default();
		let location = path;
		let files = HashSet::default();
		Datapack {
			location,
			name,
			child,
			files,
		}
	}
}

use std::path::Path;
impl From<&Path> for Datapack {
	fn from(path: &Path) -> Datapack {
		let name = os_str_to_string(path.as_os_str());
		let child = HashSet::default();
		let location = path.to_path_buf();
		let files = HashSet::default();
		Datapack {
			location,
			name,
			child,
			files,
		}
	}
}

use crate::DatapackLoader;
impl From<DatapackLoader> for Datapack {
	fn from(loader: DatapackLoader) -> Datapack {
		let name = loader.name;
		let child = HashSet::default();
		let location = loader.path;
		let files = HashSet::default();
		Datapack {
			location,
			name,
			child,
			files,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn create_new_datapack() {
		assert_eq!(
			Datapack::new(
				"/tmp/random_location",
				"random_location",
				HashSet::default(),
				HashSet::default()
			),
			Datapack {
				location: PathBuf::from("/tmp/random_location"),
				name: String::from("random_location"),
				child: HashSet::default(),
				files: HashSet::default()
			}
		);
	}

	#[test]
	fn create_new_datapack_from_path_buf() {
		assert_eq!(
			Datapack::from(PathBuf::from("/tmp/ZA_WARUDO")),
			Datapack {
				name: String::from("ZA_WARUDO"),
				location: PathBuf::from("/tmp/ZA_WARUDO"),
				child: HashSet::default(),
				files: HashSet::default()
			}
		);
	}
}
