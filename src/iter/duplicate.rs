use crate::datapack::Datapack;
use std::collections::BTreeSet;
use std::hash::Hash;
use std::iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator};

/// Iterator that iterate over all duplicated identifiers from multiple datapacks.
#[derive(Debug, Clone)]
pub struct Duplicate<'datapack, K: Ord, V, I> {
    datapacks: &'datapack [Datapack<K, V>],
    identifiers: I,
}

impl<'a, K: Ord, V, I> Duplicate<'a, K, V, I> {
    pub fn new(datapacks: &'a [Datapack<K, V>], identifiers: I) -> Self {
        Self {
            datapacks,
            identifiers,
        }
    }
}

pub fn from_datapacks<K, V>(dps: &[Datapack<K, V>]) -> Duplicate<K, V, impl Iterator<Item = &K>>
where
    K: Eq + Hash + Ord,
{
    let identifiers = dps
        .iter()
        .flat_map(|d| d.identifiers())
        .collect::<BTreeSet<_>>()
        .into_iter();
    Duplicate {
        datapacks: dps,
        identifiers,
    }
}

impl<'d, K, V, I> Iterator for Duplicate<'d, K, V, I>
where
    K: Eq + Hash + Ord,
    I: Iterator<Item = &'d K>,
{
    type Item = (&'d K, Vec<&'d V>);

    fn next(&mut self) -> Option<Self::Item> {
        let ident = self.identifiers.next()?;
        let resource = self.datapacks.iter().filter_map(|d| d.get(ident)).collect();
        let result = (ident, resource);
        Some(result)
    }
}

impl<'d, K, V, I> FusedIterator for Duplicate<'d, K, V, I>
where
    K: Eq + Hash + Ord,
    I: FusedIterator<Item = &'d K>,
{
}

impl<'a, K, V, I> DoubleEndedIterator for Duplicate<'a, K, V, I>
where
    K: Eq + Hash + Ord,
    I: DoubleEndedIterator<Item = &'a K>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let ident = self.identifiers.next_back()?;
        let resource = self.datapacks.iter().filter_map(|d| d.get(ident)).collect();
        let result = (ident, resource);
        Some(result)
    }
}

impl<'a, K, V, I> ExactSizeIterator for Duplicate<'a, K, V, I>
where
    K: Eq + Hash + Ord,
    I: ExactSizeIterator<Item = &'a K>,
{
    fn len(&self) -> usize {
        self.identifiers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn find_duplicate() {
        let alpha_datapack = {
            let mut map = BTreeMap::new();
            map.insert("/hello/world", "...");
            map.insert("/foo/bar", "...");
            Datapack::new(map)
        };

        let beta_datapack = {
            let mut map = BTreeMap::new();
            map.insert("/hello/world", "___");
            map.insert("/foo/baz", "___");
            Datapack::new(map)
        };

        let gamma_datapack = {
            let mut map = BTreeMap::new();
            map.insert("/hello/world", "^^^");
            map.insert("/baz/bar", "^^^");
            Datapack::new(map)
        };

        let datapacks = &[alpha_datapack, beta_datapack, gamma_datapack];
        let duplicates = from_datapacks(datapacks);

        let result: Vec<_> = duplicates.collect();
        let expect = vec![
            (&"/baz/bar", vec![&"^^^"]),
            (&"/foo/bar", vec![&"..."]),
            (&"/foo/baz", vec![&"___"]),
            (&"/hello/world", vec![&"...", &"___", &"^^^"]),
        ];

        assert_eq!(result, expect);
    }
}
