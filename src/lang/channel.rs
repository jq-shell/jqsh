use std::thread;
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
    /// Returns the context passed to this channel, waiting until made available by the sending end.
    pub fn context(&mut self) -> Context {
        self.context.get()
    }

    /// Takes the receiving end of a channel, asynchronously runs it through a filter, and returns the output channel.
    pub fn filter(self, f: Filter) -> Receiver {
        let (tx, rx) = channel();
        thread::spawn(move || f.run(self, tx));
        rx
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
