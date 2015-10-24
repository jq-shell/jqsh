use std::{mem, thread};

use chan;

use eventual::{self, Async};

use lang::{Context, Filter, Value};

pub struct Sender {
    pub context: eventual::Complete<Context, ()>,
    pub values: chan::Sender<Value>
    //TODO namespaces
}

pub struct Receiver {
    pub context: eventual::Future<Context, ()>,
    pub values: chan::Receiver<Value>
    //TODO namespaces
}

impl Receiver {
    /// A closed receiver with no values.
    pub fn empty(context: Context) -> Receiver {
        let (_, val_rx) = chan::async();
        Receiver {
            context: eventual::Future::of(context),
            values: val_rx
        }
    }

    /// Takes the receiving end of a channel, asynchronously runs it through a filter, and returns the output channel.
    pub fn filter(self, f: &Filter) -> Receiver {
        let (tx, rx) = channel();
        let f = f.clone();
        thread::spawn(move || f.run(self, tx));
        rx
    }

    /// Same as `filter` but waits until the filter function returns.
    pub fn filter_sync(self, f: &Filter) -> Receiver {
        let (tx, rx) = channel();
        f.run(self, tx);
        rx
    }

    /// Asynchronously forwards all received values to `dst` and closes `self`'s value channel.
    ///
    /// Returns `dst`'s context future sender.
    pub fn forward_values(&mut self, dst: Sender) -> eventual::Complete<Context, ()> {
        let (_, val_rx) = chan::async();
        let vals_to_forward = mem::replace(&mut self.values, val_rx);
        let Sender { context, values } = dst;
        thread::spawn(move || {
            for val in vals_to_forward {
                values.send(val);
            }
        });
        context
    }

    /// Split `self` into two new receivers, forwarding everything to both.
    pub fn split(self) -> (Receiver, Receiver) {
        let (Sender { context: ctxt_tx1, values: val_tx1 }, rx1) = channel();
        let (Sender { context: ctxt_tx2, values: val_tx2 }, rx2) = channel();
        let Receiver { context, values } = self;
        thread::spawn(move || {
            let context = context.await().expect("failed to split contexts");
            ctxt_tx1.complete(context.clone());
            ctxt_tx2.complete(context);
        });
        thread::spawn(move || {
            for val in values {
                val_tx1.send(val.clone());
                val_tx2.send(val);
            }
        });
        (rx1, rx2)
    }
}

impl IntoIterator for Receiver {
    type Item = Value;
    type IntoIter = chan::Iter<Value>;

    fn into_iter(self) -> chan::Iter<Value> {
        self.values.into_iter()
    }
}

pub fn channel() -> (Sender, Receiver) {
    let (ctxt_tx, ctxt_fut) = eventual::Future::pair();
    let (val_tx, val_rx) = chan::async();
    let tx = Sender {
        context: ctxt_tx,
        values: val_tx
    };
    let rx = Receiver {
        context: ctxt_fut,
        values: val_rx
    };
    (tx, rx)
}
