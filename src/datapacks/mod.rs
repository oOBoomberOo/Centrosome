mod datapack;
mod namespace;
mod script;
mod script_type;
mod data_structure;

pub use datapack::Datapack;
use namespace::Namespace;
use script::Script;
use script_type::ScriptType;

#[derive(PartialEq, Eq, Debug)]
enum FileType {
	File,
	Folder,
}

trait DataHolder {
	fn data(&self) -> &Option<Vec<u8>>;

	fn file_type(&self) -> FileType {
		if self.data().is_none() {
			FileType::Folder
		} else {
			FileType::File
		}
	}

	fn get_data(&self) -> String {
		match self.data().to_owned() {
			Some(v) => String::from_utf8(v).unwrap_or_default(),
			None => String::default(),
		}
	}
}

use std::path::PathBuf;

trait Setup {
	fn new(location: impl Into<PathBuf>, data: Option<Vec<u8>>, script_type: ScriptType) -> Self;
}

use crate::utils::{get_path_name, Result};
use std::collections::HashMap;
use std::fs;

trait DataTree<T>
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
			MergeResult::new(script, size).into()
		} else {
			let data = fs::read(physical_path)?;
			let child = HashMap::default();

			let script = Self::create(name, child, Some(data), script_type);
			MergeResult::new(script, 1).into()
		}
	}
}

trait Merger {
	fn merge(&self, other: Self, key: impl Into<String>) -> GenerateResult<Self>
	where
		Self: Sized;
}

type GenerateResult<T> = Result<MergeResult<T>>;

#[derive(Debug, Clone)]
pub struct MergeResult<T> {
	pub script: T,
	pub size: u64,
	pub key: String
}

impl<T> MergeResult<T> where T: Sized {
	fn new(script: T, size: u64) -> MergeResult<T> {
		MergeResult { script, size, key: String::default() }
	}

	fn with_key(script: T, size: u64, key: impl Into<String>) -> Option<MergeResult<T>> {
		let key = key.into();
		Some(MergeResult { script, size, key })
	}
	
	fn merge(script: T, size: u64, key: impl Into<String>) -> GenerateResult<T> {
		let key = key.into();
		Ok(MergeResult { script, size, key })
	}
}

impl<T> From<MergeResult<T>> for GenerateResult<T> where T: Sized {
	fn from(merge_result: MergeResult<T>) -> GenerateResult<T> {
		Ok(merge_result)
	}
}