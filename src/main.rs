mod parser;
mod util;

use std::fs::{self, File};

fn main() {
    fs::remove_file("pdp_out/pdp.log")
        .or_else(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })
        .expect("Couldn't clear pdp_out");
    fs::remove_file("pdp_out/token_stream.txt")
        .or_else(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })
        .expect("Couldn't clear pdp_out");
    fs::remove_file("pdp_out/parse_tree.txt")
        .or_else(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })
        .expect("Couldn't clear pdp_out");
    fs::remove_file("pdp_out/ast.txt")
        .or_else(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })
        .expect("Couldn't clear pdp_out");
    fs::remove_file("pdp_out/symbol_table.txt")
        .or_else(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })
        .expect("Couldn't clear pdp_out");

    env_logger::builder()
        .format_source_path(true)
        .format_timestamp(None)
        .format_target(false)
        .target(env_logger::Target::Pipe(Box::new(
            File::create("pdp_out/pdp.log").unwrap(),
        )))
        .init();

    let mut parser = parser::Parser::new();
    if let Err(e) = parser.parse_from_file("testing.py") {
        println!("{e}");
    }
}
