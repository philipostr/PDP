mod parser;

fn main() {
    let mut parser = parser::Parser::new();
    if let Err(e) = parser.parse_from_str("var.func()") {
        println!("{e}");
    }
}
