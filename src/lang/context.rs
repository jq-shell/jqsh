use std::fmt;
use std::sync::Arc;

use lang::Filter;

#[derive(Clone, Debug)]
pub enum PrecedenceGroup {
    AndThen
}

#[derive(Clone)]
pub struct Context {
    /// A function called each time the parser constructs a new filter anywhere in the syntax tree. If it returns false, the filter is replaced with one that generates an exception.
    pub filter_allowed: Arc<Box<Fn(Filter) -> bool + Send + Sync>>,
    /// The context's operators, in decreasing precedence.
    pub operators: Vec<PrecedenceGroup>
}

impl Context {
    /// The default context for interactive shell sessions.
    pub fn interactive() -> Context {
        Context {
            filter_allowed: Arc::new(Box::new(|_| true)),
            operators: vec![PrecedenceGroup::AndThen]
        }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "Context {{ filter_allowed: [Fn(Filter) -> bool], operators: {:?} }}", self.operators)
    }
}
