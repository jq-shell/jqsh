//extern crate readline;

mod readline { // temporary readline drop-in
    use std::io;
    
    #[allow(unused_variables)]
    pub fn add_history(hist_line: &str) {
        // history? What history?
    }
    
    pub fn readline(prompt: &str) -> Option<String> {
        print!("{}", prompt);
        match io::stdin().read_line().map(|line| line.as_slice().trim_right().to_string()) {
            Err(_) => None,
            Ok(line) => if line.as_slice() == "exit" { None } else { Some(line) }
        }
    }
}

fn main() {
    while let Some(source) = readline::readline("jqsh> ") {
        readline::add_history(source.as_slice());
        //let ast = parser::parse(source);
        //println!("{}", ast.to_string());
        println!("{}", source);
    }
}
