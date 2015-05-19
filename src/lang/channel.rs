use std::{mem, thread};
use std::sync::{mpsc, Future};

use lang::{Context, Filter, Value};

#[derive(Debug)]
pub enum ChannelError<T> {
    /// An error on the underlying channel.
    SendError(mpsc::SendError<T>),
    /// This part of the channel is already closed. Occurs when calling `set_context` or `close` multiple times or when calling `send` after `close`.
    Closed
}

impl<T> From<mpsc::SendError<T>> for ChannelError<T> {
    fn from(err: mpsc::SendError<T>) -> ChannelError<T> {
        ChannelError::SendError(err)
    }
}

#[derive(Clone)]
pub struct Sender {
    context: Option<mpsc::Sender<Context>>,
    values: Option<mpsc::Sender<Value>>
}

impl Sender {
    pub fn set_context(&mut self, ctxt: Context) -> Result<(), ChannelError<Context>> {
        match self.context {
            Some(ref tx) => {
                try!(tx.send(ctxt));
            }
            None => { return Err(ChannelError::Closed); }
        }
        self.context = None;
        Ok(())
    }

    /// Send a value.
    pub fn send(&mut self, val: Value) -> Result<(), ChannelError<Value>> {
        match self.values {
            Some(ref tx) => {
                try!(tx.send(val));
            }
            None => { return Err(ChannelError::Closed); }
        }
        Ok(())
    }

    /// Terminates transmission of values. Context and namespaces are unaffected.
    pub fn close(&mut self) -> Result<(), ChannelError<Value>> {
        if self.values.is_none() {
            return Err(ChannelError::Closed);
        }
        self.values = None;
        Ok(())
    }
}

pub struct Receiver {
    context: Future<Context>,
    values: mpsc::Receiver<Value>
}

impl Receiver {
    /// A closed receiver with no values.
    pub fn empty(context: Context) -> Receiver {
        let (_, val_rx) = mpsc::channel();
        Receiver {
            context: Future::from_value(context),
            values: val_rx
        }
    }

    /// Returns the context passed to this channel, waiting until made available by the sending end.
    pub fn context(&mut self) -> Context {
        self.context.get()
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
    /// Returns a handle to the forwarding thread, whose `join` returns `Ok(())` if all values have been successfully forwarded.
    pub fn forward_values(&mut self, mut dst: Sender) -> thread::JoinHandle<Result<(), ChannelError<Value>>> {
        let (_, rx) = mpsc::channel();
        let vals = mem::replace(&mut self.values, rx);
        thread::spawn(move || vals.iter().map(|val| dst.send(val)).fold(Ok(()), Result::and))
    }

    /// Split `self` into two new receivers, forwarding everything to both.
    pub fn split(self) -> (Receiver, Receiver) {
        let (mut tx1, rx1) = channel();
        let (mut tx2, rx2) = channel();
        let Receiver { mut context, values } = self;
        let mut context_tx1 = tx1.clone();
        let mut context_tx2 = tx2.clone();
        thread::spawn(move || {
            context_tx1.set_context(context.get()).unwrap();
            context_tx2.set_context(context.into_inner()).unwrap();
        });
        thread::spawn(move || {
            for val in values {
                tx1.send(val.clone()).unwrap();
                tx2.send(val).unwrap();
            }
        });
        (rx1, rx2)
    }
}

impl Iterator for Receiver {
    type Item = Value;

    fn next(&mut self) -> Option<Value> {
        self.values.recv().ok()
    }
}

pub fn channel() -> (Sender, Receiver) {
    let (ctxt_tx, ctxt_rx) = mpsc::channel();
    let (val_tx, val_rx) = mpsc::channel();
    let tx = Sender {
        context: Some(ctxt_tx),
        values: Some(val_tx)
    };
    let rx = Receiver {
        context: Future::from_receiver(ctxt_rx),
        values: val_rx
    };
    (tx, rx)
}
