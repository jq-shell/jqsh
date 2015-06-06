use std::fmt;
use std::sync::Arc;

use unicode::UString;

use lang::parser::{self, Code};
use lang::value::{Value, Object};
use lang::channel::{Sender, Receiver, channel};

#[derive(Clone, Debug)]
pub enum Filter {
    AndThen { lhs: Box<Filter>, remaining_code: Code },
    Empty
}

impl Filter {
    pub fn run(&self, mut input: Receiver, mut output: Sender) {
        use self::Filter::*;

        match *self {
            AndThen { ref lhs, ref remaining_code } => {
                // synchronously run the left-hand filter
                let (lhs_input, mut input) = input.split();
                let mut lhs_output = lhs_input.filter_sync(&lhs);
                // parse the right-hand filter using the lhs output context
                let rhs = match parser::parse(remaining_code.clone(), lhs_output.context()) {
                    Ok(f) => f,
                    Err(_) => {
                        output.set_context(lhs_output.context()).unwrap();
                        output.send(Value::Exception(UString::from("syntax"), Object::default())).unwrap(); //TODO more useful metadata based on the error contents
                        output.close().unwrap();
                        return;
                    }
                };
                // synchronously run the right-hand filter
                let (mut rhs_in_tx, rhs_in_rx) = channel();
                input.forward_values(rhs_in_tx.clone()); // rhs receives its values from the `;;` filter's input...
                rhs_in_tx.set_context(lhs_output.context()).unwrap(); // ...and its context from the output of lhs.
                drop(lhs_output); // the values output by lhs are discarded...
                rhs.run(rhs_in_rx, output); // and rhs is run synchronously, with output directly into the `;;` filter's output.
            }
            Empty => {
                output.close().unwrap();
                output.set_context(input.context()).unwrap();
                //TODO namespaces
            }
        }
    }
}
