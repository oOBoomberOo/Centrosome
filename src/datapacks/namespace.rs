use super::{DataTree, Script};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Namespace {
	pub name: String,
	child: HashMap<String, Script>,
}

impl DataTree<Script> for Namespace {
	fn create(name: String, child: HashMap<String, Script>, _data: Option<String>) -> Namespace {
		Namespace { name, child }
	}
}
