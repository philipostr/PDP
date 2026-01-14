mod bytecode;
mod parser;
mod util;

use log::{info, warn};
use std::fs::{self, File};

fn main() {
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).ok();

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
    fs::remove_file("pdp_out/bytecode.txt")
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

    let parser = parser::Parser::new();
    let (parse_results, symbol_table) = match parser.parse_from_file("testing.py") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    info!("Emitting bytecode");
    let mut emitter = bytecode::BytecodeEmitter::new(symbol_table);
    emitter.emit(&parse_results.ast_node);
    if let Err(e) = fs::write("pdp_out/bytecode.txt", format!("{emitter}").as_bytes()) {
        eprintln!("Warning: couldn't output bytecode: {e:?}");
        warn!("couldn't output symbol bytecode: {e:?}");
    }
}
