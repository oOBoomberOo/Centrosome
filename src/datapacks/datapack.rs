use super::{DataTree, Merger, Namespace};
use crate::utils::{get_path_name, Result};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Datapack {
	location: PathBuf,
	pub name: String,
	pub namespace: HashMap<String, Namespace>,
	pub size: u64,
}

impl Datapack {
	pub fn new(name: impl Into<String>, location: impl Into<PathBuf>) -> Datapack {
		let name = name.into();
		let location = location.into();
		Datapack {
			location,
			name,
			namespace: HashMap::default(),
			size: 0,
		}
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

				let (result, count) = Namespace::generate(path).expect("tar.gz crash?");
				namespace.insert(name, result);
				size += count;
			}
		}

		let datapack = Datapack {
			location,
			name,
			namespace,
			size,
		};

		Ok(datapack)
	}

	pub fn merge(&self, other: Datapack) -> Datapack {
		let mut result_namespace = self.namespace.clone();
		let mut size = 0;

		for (key, namespace) in other.namespace {
			let (namespace, count) = match self.namespace.get(&key) {
				Some(original) => original.merge(namespace),
				None => (namespace, 1),
			};

			size += count;
			result_namespace.insert(key, namespace);
		}

		let location = other.location.to_owned();
		let name = other.name.to_owned();
		let namespace = result_namespace;

		Datapack {
			location,
			namespace,
			name,
			size,
		}
	}
}

use std::fmt;

impl fmt::Debug for Datapack {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			write!(f, "{} {:#?}", self.name, self.namespace)
		} else {
			write!(f, "{} {:?}", self.name, self.namespace)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{Datapack, HashMap, PathBuf};

	#[test]
	fn init_datapack() {
		let value = Datapack::new("Senku", "kingdom/of/science.txt");
		let expect = Datapack {
			location: PathBuf::from("kingdom/of/science.txt"),
			name: "Senku".to_string(),
			namespace: HashMap::default(),
			size: 0
		};

		assert_eq!(value, expect);
	}
}