use super::{DataTree, Merger, Script, ScriptType, GenerateResult, MergeResult, ScriptFile};
use crate::utils::{get_path_name};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Namespace {
	pub name: String,
	child: HashMap<String, Script>,
}

impl Namespace {
	fn create(name: impl Into<String>, child: HashMap<String, Script>) -> Namespace {
		let name = name.into();
		Namespace { name, child }
	}

	pub fn generate(physical_path: PathBuf) -> GenerateResult<Namespace> {
		let name = get_path_name(&physical_path);
		let mut child: HashMap<String, Script> = HashMap::default();
		let mut count = 0;

		for entry in physical_path.read_dir()? {
			if let Ok(entry) = entry {
				let path = entry.path();
				let name = get_path_name(&path);

				let script_type = match name.as_str() {
					"advancements" => ScriptType::Advancement,
					"functions" => ScriptType::Function,
					"loot_tables" => ScriptType::LootTable,
					"predicates" => ScriptType::Predicate,
					"recipes" => ScriptType::Recipe,
					"structures" => ScriptType::Structure,
					"tags" => ScriptType::Tag,
					_ => ScriptType::Unknown
				};

				if let Ok(result) = Script::generate(&path, script_type) {
					child.insert(name, result.script);
					count += result.size;
				} else {
					eprintln!("An unknown error occurs, Debug Message: '{}'", path.display());
				}
			}
		}

		let namespace = Self::create(name, child);
		MergeResult::new(namespace, count).into()
	}

	pub fn reduce(self, location: impl Into<PathBuf>) -> Vec<ScriptFile> {
		let location = location.into();
		let location = location.join(self.name);
		let current = ScriptFile::from_namespace(&location);
		let mut result: Vec<ScriptFile> = self.child
			.into_iter()
			.flat_map(|(_, script)| script.reduce(&location).into_iter())
			.collect();
		result.push(current);
		result
	}
}

impl Merger for Namespace {
	fn merge(&self, other: Namespace, key: impl Into<String>) -> GenerateResult<Namespace> {
		let mut child = self.child.clone();

		let size = other.child
			.into_iter()
			.filter_map(|(key, script)| {
				match self.child.get(&key) {
					Some(original) => original.merge(script, &key).ok(),
					None => MergeResult::with_key(script, 1, &key)
				}
			})
			.map(|merge_result| {
				child.insert(merge_result.key, merge_result.script);
				merge_result.size
			})
			.sum();

		let name = other.name.to_owned();
		let namespace = Namespace { name, child };

		MergeResult::merge(namespace, size, key)
	}
}

#[cfg(test)]
mod tests {
	use super::{HashMap, Namespace};

	#[test]
	fn init_namespace() {
		let value = Namespace::create("Megumin", HashMap::default());
		let expect = Namespace {
			name: "Megumin".to_string(),
			child: HashMap::default(),
		};

		assert_eq!(value, expect);
	}
}
