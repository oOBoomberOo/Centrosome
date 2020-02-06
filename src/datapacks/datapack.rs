use super::{Merger, Namespace, MergeResult};
use crate::utils::{get_path_name, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Datapack {
	location: PathBuf,
	meta: Vec<u8>,
	pub name: String,
	pub namespace: HashMap<String, Namespace>,
	pub size: u64,
}

impl Datapack {
	pub fn new(name: impl Into<String>, location: impl Into<PathBuf>) -> Datapack {
		let name = name.into();
		let location = location.into();
		let meta = Vec::default();
		Datapack {
			meta,
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

				if path.is_dir() {
					let result = Namespace::generate(path).expect("Unable to generate namespace");
					namespace.insert(name, result.script);
					size += result.size;
				}
			}
		}

		let meta = fs::read(location.join("pack.mcmeta"))?;

		let datapack = Datapack {
			meta,
			location,
			name,
			namespace,
			size,
		};

		Ok(datapack)
	}

	pub fn merge(&self, other: Datapack) -> Datapack {
		let mut namespace = self.namespace.clone();

		let size = other.namespace
			.into_iter()
			.filter_map(|(key, namespace)| match self.namespace.get(&key) {
				Some(original) => original.merge(namespace, &key).ok(),
				None => MergeResult::with_key(namespace, 1, &key)
			})
			.map(|merge_result| {
				namespace.insert(merge_result.key, merge_result.script);
				merge_result.size
			})
			.sum();

		let location = other.location.to_owned();
		let name = other.name.to_owned();
		let meta = other.meta;

		Datapack {
			meta,
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

impl fmt::Display for Datapack {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name)
	}
}

#[cfg(test)]
mod tests {
	use super::{Datapack, HashMap, PathBuf};

	#[test]
	fn init_datapack() {
		let value = Datapack::new("Senku", "kingdom/of/science.txt");
		let expect = Datapack {
			meta: Vec::default(),
			location: PathBuf::from("kingdom/of/science.txt"),
			name: "Senku".to_string(),
			namespace: HashMap::default(),
			size: 0
		};

		assert_eq!(value, expect);
	}
}