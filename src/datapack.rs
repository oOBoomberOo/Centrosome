use std::collections::btree_map;
use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

type Map<K, V> = BTreeMap<K, V>;
type IntoIter<K, V> = btree_map::IntoIter<K, V>;

#[derive(Debug, Default, Clone)]
pub struct Datapack<K: Ord, V> {
    resources: Map<K, V>,
}

impl<K: Ord, V> Datapack<K, V> {
    pub fn new(resources: Map<K, V>) -> Self {
        Self { resources }
    }

    pub fn identifiers(&self) -> impl Iterator<Item = &K> {
        self.resources.keys()
    }
}

impl<K: Ord, V> Deref for Datapack<K, V> {
    type Target = Map<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.resources
    }
}

impl<K: Ord, V> DerefMut for Datapack<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resources
    }
}

impl<K: Ord, V> IntoIterator for Datapack<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.resources.into_iter()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for Datapack<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let resources = iter.into_iter().collect();
        Self::new(resources)
    }
}
