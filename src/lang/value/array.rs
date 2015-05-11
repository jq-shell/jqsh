use std::hash;
use std::iter::{order, FromIterator};
use std::sync::mpsc;

use lang::channel::Receiver;
use lang::value::Value;

//TODO implement lazy arrays by also including a receiver
#[derive(Clone, Debug)]
pub struct Array<T> {
    buffer: Vec<T>
}

impl<'a, T> Array<T> {
    pub fn new() -> Array<T> {
        Array::default()
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.buffer.get(idx)
    }

    pub fn iter(&'a self) -> Iter<'a, T> {
        Iter {
            array: self,
            index: 0
        }
    }
}

impl<T> Default for Array<T> {
    fn default() -> Array<T> {
        Array { buffer: Vec::new() }
    }
}

impl<T> From<Vec<T>> for Array<T> {
    fn from(v: Vec<T>) -> Array<T> {
        Array { buffer: v }
    }
}

impl From<Receiver> for Array<Value> {
    fn from(rx: Receiver) -> Array<Value> {
        Array { buffer: rx.collect() } //TODO implement lazy arrays
    }
}

impl<T, U> FromIterator<T> for Array<U> where U: From<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Array<U> {
        Array::from(Vec::from_iter(iter.into_iter().map(U::from)))
    }
}

pub struct IntoIter<T> {
    buffer: Vec<T>,
    channel: mpsc::Receiver<T>
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.buffer.len() > 0 {
            Some(self.buffer.remove(0))
        } else {
            self.channel.recv().ok()
        }
    }
}

pub struct Iter<'a, T: 'a> {
    array: &'a Array<T>,
    index: usize
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let result = self.array.get(self.index);
        self.index += 1;
        result
    }
}

impl<T> IntoIterator for Array<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            buffer: self.buffer,
            channel: mpsc::channel().1 //TODO use the array's channel
        }
    }
}

impl<'a, T> IntoIterator for &'a Array<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        Iter {
            array: self,
            index: 0
        }
    }
}

impl<T, U> PartialEq<Array<U>> for Array<T> where T: PartialEq<U> {
    fn eq(&self, other: &Array<U>) -> bool {
        order::eq(self.iter(), other.iter())
    }
}

impl<T: Eq> Eq for Array<T> {
    // marker trait
}

impl<T: hash::Hash> hash::Hash for Array<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        // only the first element is hashed to avoid blocking on the entire array
        if let Some(v) = self.get(0) {
            v.hash(state);
        }
    }
}
