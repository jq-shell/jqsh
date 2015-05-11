use lang::channel::{Sender, Receiver};

pub enum Filter {
    Empty
}

impl Filter {
    pub fn run(&self, mut input: Receiver, mut output: Sender) {
        use self::Filter::*;

        match *self {
            Empty => {
                output.close().unwrap();
                output.set_context(input.context()).unwrap();
                //TODO namespaces
            }
        }
    }
}
