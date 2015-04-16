extern crate jqsh;
extern crate readline;

//use jqsh::lang::context;

fn main() {
    //let mut repl_context = &context::INTERACTIVE;
    while let Some(source) = readline::readline("jqsh> ") {
        readline::add_history(&source);
        //let ast = parser::parse(source);
        //println!("{}", ast.to_string());
        println!("{}", source);
    }
}
