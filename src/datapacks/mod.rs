mod datapack;
mod namespace;
mod script;

pub use datapack::Datapack;
use namespace::Namespace;
use script::Script;

#[derive(PartialEq, Eq, Debug)]
enum FileType {
	File,
	Folder,
}

trait DataHolder {
	fn data(&self) -> &Option<String>;

	fn file_type(&self) -> FileType {
		if self.data().is_none() {
			FileType::Folder
		} else {
			FileType::File
		}
	}

	fn get_data(&self) -> String {
		self.data().clone().unwrap_or_default()
	}
}

use std::path::PathBuf;

trait Setup {
	fn new(location: impl Into<PathBuf>, data: Option<String>) -> Self;
}

use crate::utils::{get_path_name, Result};
use std::collections::HashMap;
use std::fs;

trait DataTree<T>
where
	T: Sized + DataTree<Script>,
{
	fn create(name: String, child: HashMap<String, T>, data: Option<String>) -> Self
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

					let (result, child_count) = T::generate(path)?;
					child.insert(name, result);
					count += child_count;
				}
			}

			let result = Self::create(name, child, None);
			Ok((result, count))
		} else {
			let data = fs::read_to_string(physical_path)?;
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
