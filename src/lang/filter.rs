use unicode::UString;

use eventual::Async;

use lang::parser::{self, Code};
use lang::value::{Value, Object};
use lang::channel::{Sender, Receiver, channel};

#[derive(Clone, Debug)]
pub enum Filter {
    AndThen { lhs: Box<Filter>, remaining_code: Code },
    Empty
}

impl Filter {
    pub fn run(&self, input: Receiver, output: Sender) {
        use self::Filter::*;

        match *self {
            AndThen { ref lhs, ref remaining_code } => {
                // synchronously run the left-hand filter
                let (lhs_input, mut input) = input.split();
                let Receiver { context: lhs_ctxt, values: _ } = lhs_input.filter_sync(&lhs); // the values output by lhs are discarded
                // parse the right-hand filter using the lhs output context
                let lhs_ctxt = lhs_ctxt.await().expect("failed to get context of `;;` left operand");
                let rhs = match parser::parse(remaining_code.clone(), lhs_ctxt.clone()) {
                    Ok(f) => f,
                    Err(_) => {
                        let Sender { context, values } = output;
                        context.complete(lhs_ctxt);
                        values.send(Value::Exception(UString::from("syntax"), Object::default())); //TODO more useful metadata based on the error contents
                        return;
                    }
                };
                // synchronously run the right-hand filter
                let (rhs_in_tx, rhs_in_rx) = channel();
                let rhs_in_ctxt = input.forward_values(rhs_in_tx); // rhs receives its values from the `;;` filter's input...
                rhs_in_ctxt.complete(lhs_ctxt); // ...and its context from the output of lhs.
                rhs.run(rhs_in_rx, output); // finally, rhs is run synchronously, with output directly into the `;;` filter's output.
            }
            Empty => {
                let Receiver { context: in_ctxt, values: _ } = input;
                let Sender { context, values: _ } = output;
                context.complete(in_ctxt.await().expect("failed to get input context"));
            }
        }
    }
}
