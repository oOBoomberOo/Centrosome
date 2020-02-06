mod datapack;
mod namespace;
mod script;
mod script_type;
mod data_structure;
mod script_file;

mod traits;

pub use datapack::Datapack;
use namespace::Namespace;
use script::Script;
use script_type::ScriptType;
use traits::{DataHolder, DataTree, Setup, Merger};
use script_file::ScriptFile;

#[derive(PartialEq, Eq, Debug)]
pub enum FileType {
	File,
	Folder,
}

use crate::utils::Result;
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