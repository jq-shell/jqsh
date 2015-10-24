extern crate eventual;
extern crate jqsh;
extern crate readline;
extern crate unicode;

use eventual::Async;

use unicode::UString;

use jqsh::lang::{Context, Filter, channel, parser};

fn main() {
    let mut repl_context = Context::interactive();
    while let Some(source_utf8) = readline::readline("jqsh> ") {
        readline::add_history(&source_utf8);
        let source = UString::from(source_utf8);
        let filter = parser::parse(source, repl_context.clone()).unwrap_or_else(|err| {
            println!("jqsh: syntax error: {:?}", err);
            Filter::Empty
        });
        let channel::Receiver { context, values } = channel::Receiver::empty(repl_context).filter(&filter);
        repl_context = context.await().expect("failed to get repl output context");
        for value in values {
            println!("{}", value);
        }
    }
    println!("");
}
