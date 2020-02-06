mod datapack;
mod namespace;
mod script;
mod format;

pub use datapack::Datapack;
use namespace::Namespace;
use script::Script;
use format::EncodingFormat;

#[derive(PartialEq, Eq, Debug)]
enum FileType {
	File,
	Folder,
}

trait DataHolder {
	fn data(&self) -> &Option<Vec<u8>>;

	fn file_type(&self) -> FileType {
		if self.data().is_none() {
			FileType::Folder
		} else {
			FileType::File
		}
	}

	fn get_data(&self) -> String {
		match self.data().to_owned() {
			Some(v) => String::from_utf8(v).unwrap_or_default(),
			None => String::default()
		}
	}
}

use std::path::PathBuf;

trait Setup {
	fn new(location: impl Into<PathBuf>, data: Option<Vec<u8>>) -> Self;
}

use crate::utils::{get_path_name, Result};
use std::collections::HashMap;
use std::fs;

trait DataTree<T>
where
	T: Sized + DataTree<Script>,
{
	fn create(name: String, child: HashMap<String, T>, data: Option<Vec<u8>>) -> Self
	where
		Self: Sized;

	fn generate(physical_path: PathBuf) -> Result<(Self, u64)>
	where
		Self: Sized,
	{
		let name = get_path_name(&physical_path);
		if physical_path.is_dir() {
			let mut child: HashMap<String, T> = HashMap::default();
			let mut count = 0;

			for entry in physical_path.read_dir()? {
				if let Ok(entry) = entry {
					let path = entry.path();
					let name = get_path_name(&path);

					if let Ok((result, child_count)) = T::generate(path.clone()) {
						child.insert(name, result);
						count += child_count;
					}
					else {
						eprintln!("Debug: {}", path.display());
					}
				}
			}

			let result = Self::create(name, child, None);
			Ok((result, count))
		} else {
			let data = fs::read(physical_path)?;
			let child = HashMap::default();

			let result = Self::create(name, child, Some(data));
			Ok((result, 1))
		}
	}
}

trait Merger {
	fn merge(&self, other: Self) -> (Self, u64)
	where
		Self: Sized;
}
