extern crate readline;
use readline::{add_history, readline};

fn main() {
    println!("press ctrl-c to exit");
    loop {
        match readline::readline(">>> ") {
            Some(source) => {
                readline::add_history(source.as_slice());
//              let ast = parser::parse(source);
//                println!("{}", ast.to_string());
                println!("{}", source);
            },
            None => break
        }
    }
}
