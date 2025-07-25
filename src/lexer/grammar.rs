use super::tokens::{Token, TokenType};
use super::{MarkedError, Marker};
use crate::constructs::{ast::Ast, idents_map::IdentsMap};

#[derive(Debug)]
struct Grammar {
    script: &'static str,
    ast_root: Token,

}

impl Iterator for Grammar {
    type Item = Result<(), MarkedError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(()))
    }
}

impl Grammar {
    fn from_str(script: &'static str) -> Self {
        Grammar {
            ast_root: Token::new(TokenType::Program, Marker { line: 0, col: 0 }, script),
            script,
        }
    }

    /// If the `Grammar` is completed (`.next()` returned `None`), the completed
    /// `Ast` and `IdentsMap` constructs will be returned.
    /// 
    /// # Errors
    /// A `String` is returned if any of the following reasons are true:
    /// - A proper `Ast` could not be generated/returned
    /// - A proper `IdentsMap` could not be generated/returned
    /// - The `Grammar` did not reach the end yet
    fn finalize(self) -> Result<(Ast, IdentsMap), String> {
        Err("Not done yet".to_string())
    }
}
