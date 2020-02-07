use super::{Script, ScriptType, GenerateResult, MergeResult};
use crate::utils::{get_path_name};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::ffi::OsStr;

pub trait DataTree<T>
where
	T: Sized + DataTree<Script>,
{
	fn create(name: impl Into<String>, child: HashMap<String, T>, data: Option<Vec<u8>>, script_type: ScriptType) -> Self
	where
		Self: Sized;

	fn generate(physical_path: &PathBuf, script_type: ScriptType) -> GenerateResult<Self>
	where
		Self: Sized,
	{
		let name = get_path_name(&physical_path);
		if physical_path.is_dir() {
			let mut child: HashMap<String, T> = HashMap::default();

			let size = physical_path
				.read_dir()?
				.filter_map(|entry| entry.ok())
				.filter_map(|entry| {
					let path = entry.path();
					let name = get_path_name(&path);

					T::generate(&path, script_type).map(|v| (name, v)).ok()
				})
				.map(|(name, result)| {
					child.insert(name, result.script);
					result.size
				})
				.sum();

			let script = Self::create(name, child, None, script_type);
			Ok(MergeResult::new(script, size))
		} else {
			let extension = physical_path.extension();

			if extension == Some(OsStr::new("DS_Store")) {
				return Err(Box::new(GenerateError::BlacklistExtension));
			}

			let data = fs::read(physical_path)?;
			let size = data.len() as u64;
			let child = HashMap::default();

			let script = Self::create(name, child, Some(data), script_type);
			Ok(MergeResult::new(script, size))
		}
	}
}

#[derive(Debug)]
pub enum GenerateError {
	Io(std::io::Error),
	BlacklistExtension
}

use std::error;
use std::fmt;

impl fmt::Display for GenerateError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			GenerateError::Io(error) => write!(f, "{}", error),
			GenerateError::BlacklistExtension => write!(f, "Blacklisted extension"),
		}
	}
}

impl error::Error for GenerateError {
	fn description(&self) -> &str {
		match *self {
			GenerateError::Io(ref io_err) => (io_err as &dyn error::Error).description(),
			GenerateError::BlacklistExtension => "Blacklised extension",
		}
	}

	fn cause(&self) -> Option<&dyn error::Error> {
		match *self {
			GenerateError::Io(ref io_err) => Some(io_err as &dyn error::Error),
			_ => None,
		}
	}
}

use std::io;

impl From<GenerateError> for io::Error {
	fn from(err: GenerateError) -> io::Error {
		io::Error::new(io::ErrorKind::Other, err)
	}
}