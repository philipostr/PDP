mod parser;
mod utils;

fn main() {
    let mut parser = parser::Parser::new();
    if let Err(e) = parser.parse_from_file("testing.py") {
        println!("{e}");
    }
}
