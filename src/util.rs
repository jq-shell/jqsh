use lang::channel::{Sender, Receiver};
use lang::filter::Filter;

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Clone)]
pub struct Labeled<T> {
    label: String,
    value: T
}

impl<T> Labeled<T> {
    pub fn new<S: Into<String>>(label: S, value: T) -> Labeled<T> {
        Labeled {
            label: label.into(),
            value: value
        }
    }
}

impl<T> From<T> for Labeled<T> {
    fn from(value: T) -> Labeled<T> {
        Labeled::new("", value)
    }
}

impl<T> Deref for Labeled<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Labeled<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> fmt::Debug for Labeled<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.label, f)
    }
}

pub type FilterFn = Labeled<Arc<Fn(&[Filter], Receiver, Sender) + Send + Sync>>;
