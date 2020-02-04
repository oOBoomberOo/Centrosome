mod resource;

use crate::utils::Result;
pub use resource::Resource;
use std::fs::DirEntry;
use std::path::PathBuf;

pub fn traverse_directory(directory: &PathBuf, datapack: &PathBuf) -> Result<Vec<Resource>> {
	let mut result = Vec::default();
	for entry in directory.read_dir()? {
		let entry: DirEntry = entry?;
		let path = entry.path();

		if path.is_file() {
			let resource = Resource::new(path, datapack);
			result.push(resource);
		} else if path.is_dir() {
			let mut resources = traverse_directory(&path, datapack)?;
			result.append(&mut resources);
		}
	}

	Ok(result)
}
