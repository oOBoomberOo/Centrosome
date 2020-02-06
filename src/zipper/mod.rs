mod error;

pub use error::ZipperError;

use super::Datapack;
use crate::utils::{get_path_name, Result};
use indicatif::ProgressBar;
use std::fs::{create_dir_all, metadata, remove_dir_all, File};
use std::io::BufReader;
use std::path::PathBuf;
use zip::ZipArchive;

#[derive(Debug)]
pub struct Zipper {
	reader_location: PathBuf,
	progress_bar: ProgressBar,
}

impl Zipper {
	pub fn new(reader_location: PathBuf) -> Zipper {
		let progress_bar = ProgressBar::hidden();
		Zipper {
			reader_location,
			progress_bar,
		}
	}

	pub fn set_progress_bar(mut self, progress_bar: ProgressBar) -> Zipper {
		self.progress_bar = progress_bar;
		self
	}

	pub fn peak(&self, path: &str) -> (bool, bool) {
		let reader = self.reader_location.to_owned();
		if let Ok(file) = File::open(reader) {
			let reader = BufReader::new(file);
			if let Ok(mut archive) = ZipArchive::new(reader) {
				if let Ok(result) = archive.by_name(path) {
					return (result.is_file(), result.is_dir());
				}
			}
		}
		(false, false)
	}

	pub fn datapack(&self, temp_dir: &PathBuf) -> Result<Datapack> {
		if self.reader_location.is_dir() {
			let result = Datapack::generate(&self.reader_location)?;
			self.progress_bar.set_length(result.size);
			self.progress_bar.finish_and_clear();

			Ok(result)
		} else if let Ok(extract_location) = self.extract(&temp_dir) {
			let result = Datapack::generate(&extract_location)?;
			remove_dir_all(extract_location)?;

			Ok(result)
		} else {
			self.progress_bar.abandon();
			Err(Box::new(ZipperError::UnableToConvertDatapack))
		}
	}

	pub fn extract(&self, temp_dir: &PathBuf) -> Result<PathBuf> {
		let node = &self.reader_location;
		let name = get_path_name(&node);
		let location = temp_dir.join(name);
		create_dir_all(&location)?;
		let file = File::open(node)?;

		let meta = metadata(&self.reader_location)?;
		let reader = BufReader::new(file);
		let mut archive = ZipArchive::new(reader)?;
		self.progress_bar.set_length(meta.len());

		for n in 0..archive.len() {
			let mut file = archive.by_index(n)?;
			let name = file.sanitized_name();
			self.progress_bar.inc(file.compressed_size());

			let output = location.join(name);

			if file.name().ends_with('/') {
				create_dir_all(output)?;
			} else {
				if let Some(parent) = output.parent() {
					create_dir_all(parent)?;
				}

				let mut writer = File::create(output)?;
				std::io::copy(&mut file, &mut writer)?;
			}
		}

		self.progress_bar.finish();

		Ok(location)
	}
}

impl PartialEq for Zipper {
	fn eq(&self, other: &Zipper) -> bool {
		self.reader_location == other.reader_location
	}
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;
	use super::{Zipper, ProgressBar};

	#[test]
	fn init_zipper() {
		assert_eq!(
			Zipper::new(PathBuf::from("ohayou_sekai.txt")),
			Zipper {
				reader_location: PathBuf::from("ohayou_sekai.txt"),
				progress_bar: ProgressBar::hidden()
			}
		);
	}
}