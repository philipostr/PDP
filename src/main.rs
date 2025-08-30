mod parser;
mod util;

use std::fs::File;

fn main() {
    env_logger::builder()
        .format_source_path(true)
        .format_timestamp(None)
        .format_target(false)
        .target(env_logger::Target::Pipe(Box::new(File::create("pdp.log").unwrap())))
        .init();

    let mut parser = parser::Parser::new();
    if let Err(e) = parser.parse_from_file("testing.py") {
        println!("{e}");
    }
}
