use std::hash;
use std::collections::HashMap;
use std::iter::FromIterator;

use linked_hash_map::{self, LinkedHashMap};

//TODO implement lazy objects by also including a receiver
#[derive(Clone, Debug)]
pub struct Object<K: Eq + hash::Hash, V> {
    buffer: LinkedHashMap<K, V>
}

impl<K: Eq + hash::Hash, V> Object<K, V> {
    pub fn get_idx(&self, idx: usize) -> Option<(&K, &V)> {
        self.buffer.iter().nth(idx)
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.buffer.insert(k, v)
    }

    pub fn iter(&self) -> linked_hash_map::Iter<K, V> {
        self.buffer.iter()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

impl<K: Eq + hash::Hash, V> Default for Object<K, V> {
    fn default() -> Object<K, V> {
        Object { buffer: LinkedHashMap::default() }
    }
}

impl<K: Eq + hash::Hash, L: Eq + hash::Hash, V, W> FromIterator<(K, V)> for Object<L, W> where K: Into<L>, V: Into<W> {
    fn from_iter<I: IntoIterator<Item=(K, V)>>(iter: I) -> Object<L, W> {
        let mut result = Object::default();
        for (k, v) in iter {
            result.insert(k.into(), v.into());
        }
        result
    }
}

impl<K: Eq + hash::Hash, V: hash::Hash> hash::Hash for Object<K, V> {
    fn hash<H: hash::Hasher>(&self, _: &mut H) {
        //TODO
    }
}

impl<K: Eq + hash::Hash, V: PartialEq> PartialEq<Object<K, V>> for Object<K, V> {
    fn eq(&self, other: &Object<K, V>) -> bool {
        let lhs = self.buffer.iter().collect::<HashMap<&K, &V>>();
        let rhs = other.buffer.iter().collect::<HashMap<&K, &V>>();
        lhs == rhs
    }
}

impl<K: Eq + hash::Hash, V: Eq> Eq for Object<K, V> {}

impl<K: Eq + hash::Hash + Clone, V> IntoIterator for Object<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            buffer: self.buffer
        }
    }
}

pub struct IntoIter<K: Eq + hash::Hash, V> {
    buffer: LinkedHashMap<K, V>
}

impl<K: Eq + hash::Hash + Clone, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        let next_key = match self.buffer.keys().next() {
            Some(k) => k.clone(),
            None => { return None; }
        };
        let next_value = self.buffer.remove(&next_key).unwrap();
        Some((next_key, next_value))
    }
}

impl<'a, K: Eq + hash::Hash, V> IntoIterator for &'a Object<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        Iter {
            object: self,
            index: 0
        }
    }
}

pub struct Iter<'a, K: 'a + Eq + hash::Hash, V: 'a> {
    object: &'a Object<K, V>,
    index: usize
}

impl<'a, K: Eq + hash::Hash, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        let result = self.object.get_idx(self.index);
        self.index += 1;
        result
    }
}
