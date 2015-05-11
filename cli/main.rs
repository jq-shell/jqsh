extern crate jqsh;
extern crate readline;
extern crate unicode;

use unicode::UString;

use jqsh::lang::{channel, context, parser};
use jqsh::lang::Filter;

fn main() {
    let mut repl_context = context::INTERACTIVE;
    while let Some(source_utf8) = readline::readline("jqsh> ") {
        readline::add_history(&source_utf8);
        let mut source = UString::from(source_utf8);
        loop {
            let (filter, next_source) = parser::parse(source, repl_context.clone()).unwrap_or_else(|err| {
                println!("jqsh: syntax error: {:?}", err);
                (Filter::Empty, None)
            });
            let (mut tx, rx) = channel::channel();
            let mut output = rx.filter(filter);
            tx.set_context(repl_context).unwrap();
            tx.close().unwrap();
            repl_context = output.context();
            for value in output {
                println!("{}", value);
            }
            if next_source.is_some() {
                source = next_source.unwrap();
            } else {
                break;
            }
        }
    }
    println!("");
}
