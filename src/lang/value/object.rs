use std::{hash, mem, vec};
use std::iter::FromIterator;

//TODO implement lazy objects by also including a receiver
#[derive(Clone, Debug)]
pub struct Object<K: Eq, V> {
    buffer: Vec<(K, V)>
}

impl<K: Eq, V> Object<K, V> {
    pub fn get_idx(&self, idx: usize) -> Option<(&K, &V)> {
        if self.buffer.len() > idx {
            let (ref k, ref v) = self.buffer[idx];
            Some((k, v))
        } else {
            None
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        for &mut (ref key, ref mut val) in &mut self.buffer {
            if *key == k {
                return Some(mem::replace(val, v));
            }
        }
        self.buffer.push((k, v));
        None
    }

    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            object: self,
            index: 0
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

impl<K: Eq, V> Default for Object<K, V> {
    fn default() -> Object<K, V> {
        Object { buffer: Vec::default() }
    }
}

impl<K: Eq, L: Eq, V, W> FromIterator<(K, V)> for Object<L, W> where K: Into<L>, V: Into<W> {
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

impl<K: Eq, V: PartialEq> PartialEq<Object<K, V>> for Object<K, V> {
    fn eq(&self, other: &Object<K, V>) -> bool {
        self.buffer.len() == other.buffer.len() &&
        self.iter().all(|(k1, v1)| other.iter().find(|&(k2, _)| k1 == k2).map_or(false, |(_, v2)| v2 == v1))
    }
}

impl<K: Eq, V: Eq> Eq for Object<K, V> {}

impl<K: Eq + Clone, V> IntoIterator for Object<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            buffer: self.buffer.into_iter()
        }
    }
}

pub struct IntoIter<K: Eq, V> {
    buffer: vec::IntoIter<(K, V)>
}

impl<K: Eq, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        self.buffer.next()
    }
}

impl<'a, K: Eq, V> IntoIterator for &'a Object<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Iter<'a, K, V> {
        Iter {
            object: self,
            index: 0
        }
    }
}

pub struct Iter<'a, K: 'a + Eq, V: 'a> {
    object: &'a Object<K, V>,
    index: usize
}

impl<'a, K: Eq, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        let result = self.object.get_idx(self.index);
        self.index += 1;
        result
    }
}
