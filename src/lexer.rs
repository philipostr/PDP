mod grammar;
mod tokens;

use std::fmt::Display;

#[derive(Debug)]
struct Marker {
    pub line: u64,
    pub col: u64,
}

impl Display for Marker {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.write_str(&format!("Line: {}, Col: {}", self.line, self.col))
  }
}

#[derive(Debug)]
struct MarkedError {
    marker: Marker,
    msg: String,
}
