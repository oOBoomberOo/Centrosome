#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptType {
	Advancement,
	Function,
	Predicate,
	LootTable,
	Recipe,
	Tag,
	Structure,
	Unknown
}