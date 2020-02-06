use super::ScriptType;
use std::path::PathBuf;

pub trait Setup {
	fn new(location: impl Into<PathBuf>, data: Option<Vec<u8>>, script_type: ScriptType) -> Self;
}