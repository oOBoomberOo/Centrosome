mod data_structure;
mod datapack;
mod namespace;
mod script;

use data_structure::Tag;
pub use datapack::Datapack;
use namespace::Namespace;
use script::Script;

type GeneratedResult<T> = Result<(T, u64), TreeError>;
type MergedResult<T> = Result<T, TreeError>;
type CompiledResult<T> = Result<T, TreeError>;

use std::fs::{DirEntry, File};
use zip::write::FileOptions;
use zip::ZipWriter;
/// A trait for handling recursive structure of file system
trait DataTree {
	/// Walk through files and directories and return encoded version of it
	///
	/// `event` will run when it found a file and will have that file's size as argument
	fn generate(
		entry: DirEntry,
		kind: ScriptKind,
		event: impl Fn(u64) + Copy,
	) -> GeneratedResult<Self>
	where
		Self: Sized;
	/// Merge two files or directories together
	///
	/// `event` will run when it found a file and will have that file's size as argument
	fn merge(&self, other: Self, event: impl Fn(u64) + Copy) -> MergedResult<Self>
	where
		Self: Sized;
	/// Compile the data tree down into a single zip file
	///
	/// `event` will run when it found a file and will have that file's size as argument
	fn compile(
		&self,
		path: impl Into<PathBuf>,
		zip: &mut ZipWriter<File>,
		options: &FileOptions,
		event: impl Fn(u64) + Copy,
	) -> CompiledResult<()>;
}

/// Possible type of file inside `Namespace`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptKind {
	Tag,
	Generic,
	None,
}

impl Default for ScriptKind {
	fn default() -> ScriptKind {
		ScriptKind::None
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileType {
	File(Vec<u8>),
	Directory,
}

use colored::*;
use std::io::Error;
use std::path::PathBuf;

/// Error struct for `DataTree` trait
#[derive(Debug)]
pub enum TreeError {
	Io(Error),
	Json(serde_json::Error, String),
	ZipError(zip::result::ZipError),
	FileInNamespace(PathBuf),
	FileInDatapack(PathBuf),
	UnknownFormat(String),
	MismatchType(String, String),
}

use std::fmt;
impl fmt::Display for TreeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			TreeError::Io(error) => write!(f, "{}", error),
			TreeError::Json(error, name) => write!(f, "{} in '{}'", error, name),
			TreeError::ZipError(error) => write!(f, "{}", error),
			TreeError::FileInNamespace(source) => write!(
				f,
				"'{}' is inside namespace directory. Skipping...",
				source.display().to_string().cyan()
			),
			TreeError::FileInDatapack(source) => write!(
				f,
				"'{}' is inside datapack directory. Skipping...",
				source.display().to_string().cyan()
			),
			TreeError::UnknownFormat(source) => {
				write!(f, "'{}' contained unknown format", source.cyan())
			}
			TreeError::MismatchType(source, other) => write!(
				f,
				"'{}' and '{}' somehow have different type in merging progress",
				source.cyan(),
				other.cyan()
			),
		}
	}
}

impl From<Error> for TreeError {
	fn from(error: Error) -> TreeError {
		TreeError::Io(error)
	}
}

impl From<(serde_json::Error, String)> for TreeError {
	fn from((error, name): (serde_json::Error, String)) -> TreeError {
		TreeError::Json(error, name)
	}
}

impl From<zip::result::ZipError> for TreeError {
	fn from(error: zip::result::ZipError) -> TreeError {
		TreeError::ZipError(error)
	}
}
