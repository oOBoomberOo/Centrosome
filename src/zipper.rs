use crate::datapack::Resource;
use crate::utils::Result;
use indicatif::ProgressBar;
use std::fs::File;
use std::io::{BufReader, Read};
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

	pub fn extract(&self) -> Vec<Resource> {
		use crate::datapack::traverse_directory;

		let mut result = Vec::default();

		if self.reader_location.is_dir() {
			result = traverse_directory(&self.reader_location, &self.reader_location).unwrap_or_default();
		} else if self.reader_location.is_file() {
			let reader = self.reader_location.to_owned();
			let file = File::open(reader).expect("Unable to open file");
			let reader = BufReader::new(file);
			let mut archive = ZipArchive::new(reader).expect("Unable to extract file");
			self.progress_bar.set_length(archive.len() as u64);

			for n in 0..archive.len() {
				self.progress_bar.inc(1);

				let mut file = archive
					.by_index(n)
					.expect("An error occur while extracting file");
				if file.is_file() {
					let path = file.sanitized_name();
					let mut data = String::new();
					file.read_to_string(&mut data).unwrap_or_default();
					let resource = Resource::from_data(&data, path);

					result.push(resource);
				}
			}
		}

		self.progress_bar.finish();

		result
	}
}
