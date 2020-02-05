use super::Datapack;
use crate::utils::{get_path_name, Result};
use flate2::read::GzDecoder;
use indicatif::ProgressBar;
use std::fs::{create_dir_all, metadata, remove_dir_all, File};
use std::io::BufReader;
use std::path::PathBuf;
use tar::Archive;
use zip::ZipArchive;

#[derive(Debug)]
pub struct Zipper {
	reader_location: PathBuf,
	progress_bar: ProgressBar,
	format: CompressionFormat,
}

impl Zipper {
	pub fn new(reader_location: PathBuf, format: CompressionFormat) -> Zipper {
		let progress_bar = ProgressBar::hidden();
		Zipper {
			reader_location,
			progress_bar,
			format,
		}
	}

	pub fn set_progress_bar(mut self, progress_bar: ProgressBar) -> Zipper {
		self.progress_bar = progress_bar;
		self
	}

	pub fn peak(&self, path: &str, temp_dir: &PathBuf) -> (bool, bool) {
		let reader = self.reader_location.to_owned();
		if let Ok(file) = File::open(reader) {
			match self.format {
				CompressionFormat::Zip => {
					let reader = BufReader::new(file);
					if let Ok(mut archive) = ZipArchive::new(reader) {
						if let Ok(result) = archive.by_name(path) {
							return (result.is_file(), result.is_dir());
						}
					}
				}
				CompressionFormat::Tar => {
					let temp = temp_dir.join(path);
					create_dir_all(&temp).expect("Unable to create temporary folder");

					let tar = GzDecoder::new(file);
					let mut archive = Archive::new(tar);

					let result: Vec<(bool, bool)> = archive
						.entries()
						.unwrap()
						.filter_map(|entry| entry.ok())
						.filter_map(|mut entry| -> Option<(bool, bool)> {
							let relative_path = entry.path().unwrap().to_owned();
							if PathBuf::from(path) == relative_path {
								let location = temp.join(&relative_path);
								entry.unpack(&location).unwrap();

								Some((location.is_file(), location.is_dir()))
							} else {
								None
							}
						})
						.collect();
					let result = match result.iter().next() {
						Some(value) => value.to_owned(),
						None => (false, false),
					};

					remove_dir_all(&temp).expect("Unable to remove temporary folder");

					return result;
				}
				_ => (),
			}
		}
		(false, false)
	}

	pub fn datapack(&self, temp_dir: &PathBuf) -> Result<Datapack> {
		if self.reader_location.is_dir() {
			let result = Datapack::generate(&self.reader_location).unwrap_or_default();
			self.progress_bar.set_length(result.size);
			self.progress_bar.finish_and_clear();

			Ok(result)
		} else if let Ok(extract_location) = self.extract(&temp_dir) {
			let result = Datapack::generate(&extract_location).unwrap_or_default();
			remove_dir_all(extract_location).unwrap();
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

		match self.format {
			CompressionFormat::Zip => self.extract_zip(file, &location)?,
			CompressionFormat::Tar => self.extract_tar(file, &location)?,
			_ => (),
		}

		Ok(location)
	}

	// TODO: Progress bar for tarball
	fn extract_tar(&self, file: File, location: &PathBuf) -> Result<()> {
		use tar::Entry;

		let tar = GzDecoder::new(file);
		let mut archive = Archive::new(tar);
		self.progress_bar.set_length(1);

		archive.entries()?.filter_map(|entry| entry.ok()).for_each(
			|mut entry: Entry<GzDecoder<File>>| {
				if let Ok(relative_path) = entry.path() {
					let path = location.join(relative_path);
					let parent = path.parent().unwrap();
					create_dir_all(parent).unwrap();
					entry.unpack(&path).unwrap();
					let header = entry.header();

					self.progress_bar
						.inc(header.entry_size().unwrap_or_default());
				}
			},
		);

		self.progress_bar.finish_at_current_pos();

		Ok(())
	}

	fn extract_zip(&self, file: File, location: &PathBuf) -> Result<()> {
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

		Ok(())
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CompressionFormat {
	Zip,
	Tar,
	Directory,
	Unknown(PathBuf),
}

#[derive(Debug)]
pub enum ZipperError {
	Io(std::io::Error),
	UnableToConvertDatapack,
	UnknownFormat(PathBuf),
}

use std::error;
use std::fmt;

impl fmt::Display for ZipperError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ZipperError::Io(error) => write!(f, "{}", error),
			ZipperError::UnableToConvertDatapack => write!(f, "Unable to convert to datapack"),
			ZipperError::UnknownFormat(path) => {
				write!(f, "Unknown file format: {}", path.display())
			}
		}
	}
}

impl error::Error for ZipperError {
	fn description(&self) -> &str {
		match *self {
			ZipperError::Io(ref io_err) => (io_err as &dyn error::Error).description(),
			ZipperError::UnableToConvertDatapack => "Unable to convert to datapack",
			ZipperError::UnknownFormat(_) => "Unknown file format",
		}
	}

	fn cause(&self) -> Option<&dyn error::Error> {
		match *self {
			ZipperError::Io(ref io_err) => Some(io_err as &dyn error::Error),
			_ => None,
		}
	}
}

use std::convert;
use std::io;

impl convert::From<ZipperError> for io::Error {
	fn from(err: ZipperError) -> io::Error {
		io::Error::new(io::ErrorKind::Other, err)
	}
}
