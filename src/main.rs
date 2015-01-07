extern crate readline;

fn main() {
    while let Some(source) = readline::readline("jqsh> ") {
        readline::add_history(source.as_slice());
        //let ast = parser::parse(source);
        //println!("{}", ast.to_string());
        println!("{}", source);
    }
}
