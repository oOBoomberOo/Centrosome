use crate::utils::os_str_to_string;
use std::fs;
use std::fs::File;
use std::fs::Metadata;
use std::io::Result;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use zip::read::ZipFile;
use zip::ZipArchive;

/// Abstraction layer for datapack
/// 
/// Because datapack can come in either 'directory' or 'zip file' format
#[derive(Clone, Debug)]
pub struct DatapackLoader {
	pub path: PathBuf,
	pub name: String,
	is_temp: bool,
}

impl DatapackLoader {
	pub fn cleanup(&self) {
		if self.is_temp {
			fs::remove_dir_all(&self.path).unwrap();
		}
	}

	pub fn new(origin: impl Into<PathBuf>) -> Result<DatapackLoader> {
		let origin = origin.into();
		let name = os_str_to_string(origin.file_name().unwrap());
		if origin.is_file() {
			let path = DatapackLoader::extract(origin)?;
			Ok(DatapackLoader {
				path,
				name,
				is_temp: true,
			})
		} else {
			Ok(DatapackLoader {
				path: origin,
				name,
				is_temp: false,
			})
		}
	}

	/// Get metadata of a single file without having to loop through the entire datapack
	pub fn peak(origin: &PathBuf, path: &str) -> Result<Metadata> {
		if origin.is_file() {
			let file = File::open(&origin)?;
			let mut zip = ZipArchive::new(file)?;

			let temp_dir = tempdir()?;
			let file = zip.by_name(path)?;
			let path = DatapackLoader::materialize_reader(file, temp_dir.path())?;
			fs::metadata(path)
		} else {
			origin.join(path).metadata()
		}
	}

	fn extract(origin: PathBuf) -> Result<PathBuf> {
		let directory = tempdir()?;

		let file = File::open(&origin)?;
		let mut zip = ZipArchive::new(file)?;
		for n in 0..zip.len() {
			let file = zip.by_index(n)?;
			DatapackLoader::materialize_reader(file, directory.path())?;
		}

		Ok(directory.into_path())
	}

	fn materialize_reader(mut reader: ZipFile, output: &Path) -> Result<PathBuf> {
		let location = output.join(reader.sanitized_name());
		if reader.is_dir() {
			fs::create_dir_all(&location)?;
		} else {
			if let Some(parent) = location.parent() {
				fs::create_dir_all(parent)?;
			}

			let mut writer = File::create(&location).unwrap();
			std::io::copy(&mut reader, &mut writer)?;
		}

		Ok(location)
	}
}
