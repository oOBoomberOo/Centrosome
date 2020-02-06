use super::{DataTree, Merger, Script};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Namespace {
	pub name: String,
	child: HashMap<String, Script>,
}

impl DataTree<Script> for Namespace {
	fn create(name: impl Into<String>, child: HashMap<String, Script>, _data: Option<Vec<u8>>) -> Namespace {
		let name = name.into();
		Namespace { name, child }
	}
}

impl Merger for Namespace {
	fn merge(&self, other: Namespace) -> (Namespace, u64) {
		let mut result_child = self.child.clone();
		let mut counts = 0;

		for (key, script) in other.child {
			let (script, count) = match self.child.get(&key) {
				Some(original) => original.merge(script),
				None => (script, 1),
			};

			counts += count;
			result_child.insert(key, script);
		}

		let name = other.name.to_owned();
		let child = result_child;

		(Namespace { name, child }, counts)
	}
}

#[cfg(test)]
mod tests {
	use super::{Namespace, DataTree, HashMap};

	#[test]
	fn init_namespace() {
		let value = Namespace::create("Megumin", HashMap::default(), None);
		let expect = Namespace {
			name: "Megumin".to_string(),
			child: HashMap::default()
		};

		assert_eq!(value, expect);
	}
}