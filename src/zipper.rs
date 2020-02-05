use super::Datapack;
use crate::utils::{get_path_name, Result};
use indicatif::ProgressBar;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::{BufReader};
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

	pub fn datapack(&self) -> Result<Datapack> {
		if self.reader_location.is_dir() {
			self.progress_bar.finish();
			Ok(Datapack::generate(&self.reader_location).unwrap_or_default())
		} else if let Ok(extract_location) = self.extract() {
			let result = Datapack::generate(&extract_location).unwrap_or_default();
			remove_dir_all(extract_location).unwrap();
			Ok(result)
		} else {
			self.progress_bar.abandon();
			Err(Box::new(ZipperError::UnableToConvertDatapack))
		}
	}
	pub fn extract(&self) -> Result<PathBuf> {
		let node = &self.reader_location;
		let name = get_path_name(&node);
		let location = std::env::temp_dir().join(name);
		create_dir_all(&location)?;
		let file = File::open(node)?;
		let reader = BufReader::new(file);
		let mut archive = ZipArchive::new(reader)?;
		self.progress_bar.set_length(archive.len() as u64);

		for n in 0..archive.len() {
			let mut file = archive.by_index(n)?;
			let name = file.sanitized_name();
			self.progress_bar.inc(1);

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

#[derive(Debug)]
pub enum ZipperError {
	Io(std::io::Error),
	UnableToConvertDatapack,
}

use std::error;
use std::fmt;

impl fmt::Display for ZipperError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ZipperError::Io(error) => write!(f, "{}", error),
			ZipperError::UnableToConvertDatapack => write!(f, "Unable to convert to datapack"),
		}
	}
}

impl error::Error for ZipperError {
	fn description(&self) -> &str {
		match *self {
			ZipperError::Io(ref io_err) => (io_err as &dyn error::Error).description(),
			ZipperError::UnableToConvertDatapack => "Unable to convert to datapack",
		}
	}

	fn cause(&self) -> Option<&dyn error::Error> {
		match *self {
			ZipperError::Io(ref io_err) => Some(io_err as &dyn error::Error),
			_ => None,
		}
	}
}

use std::io;
use std::convert;

impl convert::From<ZipperError> for io::Error
{
    fn from(err: ZipperError) -> io::Error
    {
        io::Error::new(io::ErrorKind::Other, err)
    }
}