use super::GenerateResult;

pub trait Merger {
	fn merge(&self, other: Self, key: impl Into<String>) -> GenerateResult<Self>
	where
		Self: Sized;
}