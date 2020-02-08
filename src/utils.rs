use crate::datapacks::TreeError;
use std::io;
use std::path::Path;

pub type DatapackIterator = Box<dyn Iterator<Item = DirEntry>>;
pub type DatapacksResult = io::Result<DatapackIterator>;
/// Return iterator over every valid datapacks inside `directory`
pub fn get_datapacks(directory: &Path) -> DatapacksResult {
	let result = directory
		.read_dir()?
		.filter_map(|entry| check_datapack(entry).ok());
	Ok(Box::new(result))
}

use super::DatapackLoader;
use std::fs::DirEntry;
use std::io::{Error, ErrorKind};
/// Determine if `entry` is a datapack or not by checking for `/pack.mcmeta` and `/data` inside `entry`
fn check_datapack(entry: io::Result<DirEntry>) -> io::Result<DirEntry> {
	let entry = entry?;
	let path = entry.path();

	let pack_mcmeta = DatapackLoader::peak(&path, "pack.mcmeta")?;
	let data_folder = DatapackLoader::peak(&path, "data/")?;

	if is_datapack(pack_mcmeta, data_folder) {
		Ok(entry)
	} else {
		Err(Error::new(
			ErrorKind::NotFound,
			"This path is not a valid datapack",
		))
	}
}

use std::fs::Metadata;
fn is_datapack(pack_mcmeta: Metadata, data_folder: Metadata) -> bool {
	pack_mcmeta.is_file() && data_folder.is_dir()
}

use std::ffi::OsString;
/// Because Rust's string can't exactly hold the entire OsString, it need to be loosely translate first.
pub fn os_str_to_string(value: impl Into<OsString>) -> String {
	let value = value.into();
	value.to_string_lossy().into_owned()
}

#[derive(Debug)]
pub enum MergeError {
	Io(Error),
	Tree(TreeError),
	Other(&'static str),
	Cancel,
}

use std::fmt;
impl fmt::Display for MergeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			MergeError::Io(error) => write!(f, "{}", error),
			MergeError::Tree(error) => write!(f, "{}", error),
			MergeError::Other(message) => write!(f, "{}", message),
			MergeError::Cancel => write!(f, "Cancelled."),
		}
	}
}

impl From<Error> for MergeError {
	fn from(error: Error) -> MergeError {
		MergeError::Io(error)
	}
}

impl From<TreeError> for MergeError {
	fn from(error: TreeError) -> MergeError {
		MergeError::Tree(error)
	}
}

use zip::CompressionMethod;
pub fn get_compression_method() -> CompressionMethod {
	if cfg!(feature = "bzip2") {
		CompressionMethod::Bzip2
	} else if cfg!(feature = "deflate") {
		CompressionMethod::Deflated
	} else {
		CompressionMethod::Stored
	}
}
