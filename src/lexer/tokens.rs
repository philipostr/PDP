use std::collections::VecDeque;

use super::{MarkedError, Marker};

#[derive(Debug)]
pub enum TokenType {
    Program,
    Unit { in_loop: bool, in_function: bool },
    Result { in_loop: bool },
    Scoped { in_loop: bool },
    Expr,
    Iter,
    Params,
    ParamsMaybe,
    SideEffect,
    SideEffect_,
    List,
    ListHead,
    ListTail,
    Dict,
    DictHead,
    DictTail,
}

#[derive(Debug)]
pub struct Token {
    ttype: TokenType,
    marker: Marker,
    script_section: &'static str,
    children: Option<Vec<Token>>,
}

impl Token {
    pub fn marker(&self) -> &Marker {
        &self.marker
    }

    pub fn section(&self) -> &str {
        self.script_section
    }

    /// Returns a reference to the `Vec` of children that this expansion generated,
    /// if any.
    /// 
    /// # Errors
    /// A `MarkedError` is returned if no branches were valid to expand to.
    pub fn expand(&mut self) -> Result<Option<&Vec<Token>>, MarkedError> {
        Ok(None)
    }

    pub fn new(ttype: TokenType, marker: Marker, script_section: &'static str) -> Self {
        Token {
            ttype,
            marker,
            script_section,
            children: None,
        }
    }
}
