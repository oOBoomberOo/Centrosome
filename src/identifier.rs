pub trait Identifier {
    fn is_similar(&self, other: &Self) -> bool;
    /// Return true if the identifer come from exactly the same place
    fn is_same(&self, other: &Self) -> bool;
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Namespace {}

impl Identifier for Namespace {
    fn is_similar(&self, other: &Self) -> bool {
        todo!()
    }

    fn is_same(&self, other: &Self) -> bool {
        todo!()
    }
}
