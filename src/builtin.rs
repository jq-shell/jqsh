use std::sync::Arc;

use num::{FromPrimitive, BigRational};

use lang::context::{Context, PrecedenceGroup};

/// The default context for interactive shell sessions.
pub fn context() -> Context {
    Context {
        filter_allowed: Arc::new(Box::new(|_| true)),
        operators: vec![
            (1_000_000, PrecedenceGroup::Circumfix),
            (-1_000_000, PrecedenceGroup::AndThen)
        ].into_iter().map(|(precedence, group)| {
            (BigRational::from_integer(FromPrimitive::from_i32(precedence).unwrap()), group)
        }).collect()
    }
}
