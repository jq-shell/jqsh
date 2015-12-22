use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use num::BigRational;

use lang::Filter;

#[derive(Clone, Debug)]
pub enum PrecedenceGroup {
    AndThen,
    Circumfix
}

#[derive(Clone)]
pub struct Context {
    /// A function called each time the parser constructs a new filter anywhere in the syntax tree. If it returns false, the filter is replaced with one that generates an exception.
    pub filter_allowed: Arc<Box<Fn(&Filter) -> bool + Send + Sync>>,
    /// The context's operators, in decreasing precedence.
    pub operators: BTreeMap<BigRational, PrecedenceGroup>
}

impl fmt::Debug for Context {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "Context {{ filter_allowed: [Fn(&Filter) -> bool], operators: {:?} }}", self.operators)
    }
}
