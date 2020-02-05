use super::{Namespace, DataTree};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::utils::{Result, get_path_name};

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Datapack {
	location: PathBuf,
	pub name: String,
	pub namespace: HashMap<String, Namespace>,
	pub size: u64
}

impl Datapack {
	pub fn new(location: impl Into<PathBuf>) -> Datapack {
		let location = location.into();
		let name = get_path_name(&location);
		let namespace = HashMap::default();
		Datapack { location, name, namespace, size: 0 }
	}

	pub fn generate(location: impl Into<PathBuf>) -> Result<Datapack> {
		let location = location.into();
		let name = get_path_name(&location);
		let mut namespace = HashMap::default();
		let mut size = 0;

		for entry in location.join("data").read_dir()? {
			if let Ok(entry) = entry {
				let path = entry.path();
				let name = get_path_name(&path);

				let (result, count) = Namespace::generate(path)?;
				namespace.insert(name, result);
				size += count;
			}
		}

		let datapack = Datapack { location, name, namespace, size };

		Ok(datapack)
	}
}

use std::fmt;

impl fmt::Debug for Datapack {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			write!(f, "{} {:#?}", self.name, self.namespace)
		}
		else {
			write!(f, "{} {:?}", self.name, self.namespace)
		}
	}
}