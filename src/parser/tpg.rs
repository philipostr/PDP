#![allow(dead_code)]

use log::{debug, error, trace};
use std::fmt::Debug;

use super::markers::*;
use super::{ParseError, building_blocks::*};
use crate::{parser::ptag::AstNode, util::TwoWayIterator};

#[derive(Debug, Default, Clone)]
pub struct Context {
    pub indentation: usize,
    pub in_loop: bool,
    pub in_function: bool,
}

#[derive(Debug)]
pub struct Star<N: ParseTreeNode>(Vec<N>);
#[derive(Debug)]
pub struct Plus<N: ParseTreeNode>(Vec<N>);
#[derive(Debug)]
pub struct Maybe<N: ParseTreeNode>(Option<N>);

impl<N: ParseTreeNode> Star<N> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<N: ParseTreeNode> Plus<N> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<N: ParseTreeNode> Maybe<N> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }
}

/* NODE DEFINITIONS START HERE */

pub trait ParseTreeNode: Sized + Debug {
    fn parse(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>);
}

#[derive(Debug)]
pub struct OpTokenNode(Op, usize, usize);
impl OpTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::OP(op, line, col) => Self(op.clone(), *line, *col),
            t => panic!("Attempted to make `OpTokenNode` from {t:?}"),
        }
    }

    pub fn as_ast(&self) -> MarkedAstNode {
        let mark = Marker {
            row: self.1,
            col: self.2,
        };
        MarkedAstNode::new(AstNode::op(MarkedOp::new(self.0.clone(), mark)), mark)
    }
}

#[derive(Debug)]
pub struct AsopTokenNode(Asop, usize, usize);
impl AsopTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::ASOP(asop, line, col) => Self(asop.clone(), *line, *col),
            t => panic!("Attempted to make `AsopTokenNode` from {t:?}"),
        }
    }

    pub fn as_ast(&self) -> MarkedAstNode {
        let mark = Marker {
            row: self.1,
            col: self.2,
        };
        MarkedAstNode::new(AstNode::asop(MarkedAsop::new(self.0.clone(), mark)), mark)
    }
}

#[derive(Debug)]
pub struct KeywordTokenNode(Keyword, usize, usize);
impl KeywordTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::KEYWORD(kw, line, col) => Self(kw.clone(), *line, *col),
            t => panic!("Attempted to make `KeywordTokenNode` from {t:?}"),
        }
    }

    pub fn as_ast(&self) -> MarkedAstNode {
        let mark = Marker {
            row: self.1,
            col: self.2,
        };
        MarkedAstNode::new(
            AstNode::keyword(MarkedKeyword::new(self.0.clone(), mark)),
            mark,
        )
    }
}

#[derive(Debug)]
pub struct NameTokenNode(String, usize, usize);
impl NameTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::NAME(name, line, col) => Self(name.clone(), *line, *col),
            t => panic!("Attempted to make `NameTokenNode` from {t:?}"),
        }
    }

    pub fn as_ast(&self) -> MarkedAstNode {
        let mark = Marker {
            row: self.1,
            col: self.2,
        };
        MarkedAstNode::new(AstNode::name(MarkedString::new(self.0.clone(), mark)), mark)
    }
}

#[derive(Debug)]
pub struct StringTokenNode(String, usize, usize);
impl StringTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::STRING(s, line, col) => Self(s.clone(), *line, *col),
            t => panic!("Attempted to make `StringTokenNode` from {t:?}"),
        }
    }

    pub fn as_ast(&self) -> MarkedAstNode {
        let mark = Marker {
            row: self.1,
            col: self.2,
        };
        MarkedAstNode::new(
            AstNode::string(MarkedString::new(self.0.clone(), mark)),
            mark,
        )
    }
}

#[derive(Debug)]
pub struct NumberTokenNode(f64, usize, usize);
impl NumberTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::NUMBER(n, line, col) => Self(*n, *line, *col),
            t => panic!("Attempted to make `NumberTokenNode` from {t:?}"),
        }
    }

    pub fn as_ast(&self) -> MarkedAstNode {
        let mark = Marker {
            row: self.1,
            col: self.2,
        };
        MarkedAstNode::new(AstNode::number(MarkedNumber::new(self.0, mark)), mark)
    }
}

#[derive(Debug)]
pub struct BoolTokenNode(bool, usize, usize);
impl BoolTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::BOOL(b, line, col) => Self(*b, *line, *col),
            t => panic!("Attempted to make `BoolTokenNode` from {t:?}"),
        }
    }

    pub fn as_ast(&self) -> MarkedAstNode {
        let mark = Marker {
            row: self.1,
            col: self.2,
        };
        MarkedAstNode::new(AstNode::boolean(MarkedBoolean::new(self.0, mark)), mark)
    }
}

/// The entire script.
///
/// ```
/// Program: END
///        | Scoped* END
/// ```
#[derive(Debug)]
pub enum ProgramNode {
    None,
    Some(Star<ScopedNode>),
}

/// A line that is scoped with `n` indents and ends with a NEWLINE.
///
/// ```
/// Scoped: NEWLINE       
///       | INDENT{n} Unit
/// ```
#[derive(Debug)]
pub enum ScopedNode {
    None,
    Some(Box<UnitNode>),
}

/// The contents of a line, including the NEWLINE.
///
/// ```
/// Unit:  KEYWORD(If) Expr MISC(':') Result
///      | KEYWORD(While) Expr MISC(':') Result   [l = true]
///      | KEYWORD(For) NAME OP(In) Expr MISC(:) Result   [l = true]
///  [l] | KEYWORD(Continue) NEWLINE
///  [l] | KEYWORD(Break) NEWLINE
///  [f] | KEYWORD(Return) Expr? NEWLINE
///      | KEYWORD(Def) NAME BRACKET('(') Params? BRACKET(')') MISC(':') Body   [f = true]
///      | NAME SideEffect NEWLINE
/// ```
#[derive(Debug)]
pub enum UnitNode {
    If(Box<ExprNode>, Box<ResultNode>),
    While(Box<ExprNode>, Box<ResultNode>),
    For(NameTokenNode, Box<ExprNode>, Box<ResultNode>),
    Continue,
    Break,
    Return(Maybe<ExprNode>),
    Def(NameTokenNode, Maybe<ParamsNode>, Box<BodyNode>),
    Name(NameTokenNode, Box<SideEffectNode>),
}

/// A helper node to give blocks the option to be a single in-line statement.
///
/// ```
/// Result: NEWLINE Scoped+   [n += 1]
///       | NAME SideEffect NEWLINE
/// ```
#[derive(Debug)]
pub enum ResultNode {
    MultiLine(Plus<ScopedNode>),
    InLine(NameTokenNode, Box<SideEffectNode>),
}

/// A helper node to give function bodies the option to be a single in-line return statement.
///
/// ```
/// Body: NEWLINE Scoped+   [n += 1]
///     | KEYWORD(Return) Expr NEWLINE
/// ```
#[derive(Debug)]
pub enum BodyNode {
    MultiLine(Plus<ScopedNode>),
    InLine(Box<ExprNode>),
}

/// To call NAME as a function, or assign to it a value as a variable or indexed object.
///
/// ```
/// SideEffect: BRACKET('(') List? BRACKET(')')
///           | Index* ASOP Expr
/// ```
#[derive(Debug)]
pub enum SideEffectNode {
    Call(Maybe<ListNode>),
    Asop(Star<IndexNode>, AsopTokenNode, Box<ExprNode>),
}

/// Any expression that can return a value.
///
/// ```
/// Expr: ExprUnary ExprBinary*
/// ```
#[derive(Debug)]
pub struct ExprNode(Box<ExprUnaryNode>, Star<ExprBinaryNode>);

/// An expression potentially starting with a unary operation.
///
/// ```
/// ExprUnary: OP(Minus) ExprUnit
///          | OP(Not) ExprUnit
///          | ExprUnit
/// ```
#[derive(Debug)]
pub enum ExprUnaryNode {
    Minus(Box<ExprUnitNode>),
    Not(Box<ExprUnitNode>),
    Unit(Box<ExprUnitNode>),
}

/// The main container of any kind of expression.
///
/// ```
/// ExprUnit: NAME NameExpr
///         | BRACKET('(') Expr BRACKET(')')
///         | BRACKET('[') List? BRACKET(']')
///         | BRACKET('{') BracExpr? BRACKET('}')
///         | STRING
///         | NUMBER
///         | BOOLEAN
/// ```
#[derive(Debug)]
pub enum ExprUnitNode {
    Name(NameTokenNode, Box<NameExprNode>),
    Paren(Box<ExprNode>),
    Bracket(Maybe<ListNode>),
    Brace(Maybe<BracExprNode>),
    String(StringTokenNode),
    Number(NumberTokenNode),
    Bool(BoolTokenNode),
}

/// Helper node for Expr to have multiple subexpressions joined through binary operations.
///
/// ```
/// ExprBinary: OP ExprUnit
/// ```
#[derive(Debug)]
pub struct ExprBinaryNode(OpTokenNode, Box<ExprUnitNode>);

/// Helper node for ExprUnit to access a NAME in ways outside of basic value-retrieval.
///
/// ```
/// NameExpr: BRACKET('(') List? BRACKET(')')
///         | Index*
/// ```
#[derive(Debug)]
pub enum NameExprNode {
    Call(Maybe<ListNode>),
    Index(Star<IndexNode>),
}

/// Helper node for ExprUnit to create sets and dictionaries.
///
/// ```
/// BracExpr: Dict
///         | List
/// ```
#[derive(Debug)]
pub enum BracExprNode {
    Dict(Box<DictNode>),
    List(Box<ListNode>),
}

/// A comma-separated list of expressions.
///
/// ```
/// List: Expr ListTail*
/// ```
#[derive(Debug)]
pub struct ListNode(Box<ExprNode>, Star<ListTailNode>);

/// Helper node for List to have multiple values.
///
/// ```
/// ListTail: MISC(',') Expr
/// ```
#[derive(Debug)]
pub struct ListTailNode(Box<ExprNode>);

/// List but only allowing identifiers.
///
/// ```
/// Params: NAME ParamsTail*
/// ```
#[derive(Debug)]
pub struct ParamsNode(NameTokenNode, Star<ParamsTailNode>);

/// Helper node for Params to have multiple values.
///
/// ```
/// ParamsTail: MISC(',') NAME
/// ```
#[derive(Debug)]
pub struct ParamsTailNode(NameTokenNode);

/// A comma-separated list of key-value pairs.
///
/// ```
/// Dict: STRING MISC(':') Expr DictTail*
/// ```
#[derive(Debug)]
pub struct DictNode(StringTokenNode, Box<ExprNode>, Star<DictTailNode>);

/// Helper node for Dict to have multiple key-value pairs.
///
/// ```
/// DictTail: MISC(',') STRING MISC(':') Expr
/// ```
#[derive(Debug)]
pub struct DictTailNode(StringTokenNode, Box<ExprNode>);

/// The index of an indexable NAME.
///
/// ```
/// Index: BRACKET('[') Expr BRACKET(']')
/// ```
#[derive(Debug)]
pub struct IndexNode(Box<ExprNode>);

/* NODE DEFINITIONS END HERE */

/* TPG STARTS HERE */

/// `match_node!(<node type>, token_stream, context, advanced)`
macro_rules! match_node {
    ($node_type:ident, $token_stream:ident, $context:ident, $advanced:ident) => {{
        let node = $node_type::parse($token_stream, $context);
        $advanced += node.0;
        let node = match node.1 {
            Ok(n) => n,
            Err(e) => {
                trace!("[{}::parse()] {e}", stringify!($node_type));
                return ($advanced, Err(e));
            }
        };
        node
    }};
}

/// `match_meta_node!(<node type>, <quantifier>, token_stream, context, advanced)`
macro_rules! match_meta_node {
    ($node_type:ident, $quantifier:ident, $token_stream:ident, $context:ident, $advanced:ident) => {{
        let meta_node = $quantifier::<$node_type>::parse($token_stream, $context);
        $advanced += meta_node.0;
        match meta_node.1 {
            Ok(n) => n,
            Err(e) => {
                trace!("[{}::parse()] {e}", stringify!($node_type));
                return ($advanced, Err(e));
            }
        }
    }};
}

/// Return token node: `match_token!(<token pattern>, <token node struct>, <error message>, token_stream, advanced)`
///
/// Just do the match: `match_token!(<token pattern>, <error message>, token_stream, advanced)`
macro_rules! match_token {
    ($token_pat:pat, $token_node:ident, $err_message:literal, $token_stream:ident, $advanced:ident) => {{
        $advanced += 1;
        match $token_stream.next() {
            Some(t @ $token_pat) => $token_node::from_token(t),
            Some(t) => {
                trace!(
                    "[{}::parse()] {} ({t:?})",
                    stringify!($token_node),
                    $err_message
                );
                let (line, col) = t.line_and_col();
                return ($advanced, Err(ParseError::marked($err_message, line, col)));
            }
            None => {
                error!(
                    "[{}::parse()] The token stream somehow ended early",
                    stringify!($token_node)
                );
                return (
                    $advanced,
                    Err(ParseError::general(
                        "Grammar error: the token stream somehow ended early...",
                    )),
                );
            }
        }
    }};

    ($token_pat:pat, $err_message:literal, $token_stream:ident, $advanced:ident) => {{
        $advanced += 1;
        match $token_stream.next() {
            Some($token_pat) => {}
            Some(t) => {
                trace!("{} ({t:?} != {})", $err_message, stringify!($token_pat));
                let (line, col) = t.line_and_col();
                return ($advanced, Err(ParseError::marked($err_message, line, col)));
            }
            None => {
                error!("The token stream somehow ended early");
                return (
                    $advanced,
                    Err(ParseError::general(
                        "Grammar error: the token stream somehow ended early...",
                    )),
                );
            }
        }
    }};
}

pub struct ParseTokensRes<N: ParseTreeNode> {
    pub parse_node: N,
    pub ast_node: MarkedAstNode,
}

impl<N: ParseTreeNode> ParseTokensRes<N> {
    pub fn new(parse_node: N, ast_node: MarkedAstNode) -> Self {
        ParseTokensRes {
            parse_node,
            ast_node,
        }
    }
}

pub fn parse_tokens(token_stream: &Vec<Token>) -> Result<ParseTokensRes<ProgramNode>, ParseError> {
    debug!("parse_tokens() started");
    let context = Context::default();
    let mut iter = TwoWayIterator::from_source(token_stream);
    ProgramNode::parse(&mut iter, &context).1
}

impl<N: ParseTreeNode> ParseTreeNode for Star<N> {
    /// Leaves the token stream at the first unmatched token.
    ///
    /// Results in an error if, in the current node matching attempt, the next token IS matched but the entire node isn't. Otherwise, an error
    /// should come from matching the next node in the caller.
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        let type_name = std::any::type_name::<N>();
        debug!("Star::<{type_name}>::parse() started");

        let mut advanced = 0;
        let mut parse_group = Vec::new();
        let mut ast_group = Vec::new();

        // Match as many `N`s as possible before failing
        loop {
            let result = N::parse(token_stream, context);
            match result.1 {
                Ok(n) => {
                    parse_group.push(n.parse_node);
                    ast_group.push(n.ast_node);
                }
                Err(e) => {
                    // Ignore the actual error if the next token was not matched
                    if result.0 == 1 {
                        token_stream.rev();
                    // Propagate the error if the next token WAS matched
                    } else {
                        trace!(
                            "[Star::<{type_name}>::parse()] Failed on node match attempt {}",
                            parse_group.len() + 1
                        );
                        return (advanced + result.0, Err(e));
                    }
                    break;
                }
            }
            advanced += result.0;
        }

        trace!(
            "[Star::<{type_name}>::parse()] Matched {} nodes",
            parse_group.len()
        );
        let mark = if let Some(result) = ast_group.first() {
            result.mark
        } else {
            // The marker obviously doesn't matter if nothing was matched, because there's nothing to mark
            Marker::default()
        };
        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(parse_group),
                MarkedAstNode::new(AstNode::multiple(ast_group), mark),
            )),
        )
    }
}

impl<N: ParseTreeNode> ParseTreeNode for Plus<N> {
    /// Leaves the token stream at the first unmatched token.
    ///
    /// Results in an error if, in the current node matching attempt, the next token IS matched but the entire node isn't. Otherwise, an error
    /// should come from matching the next node in the caller.
    /// Also, an error is returned if no nodes are matched.
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        let type_name = std::any::type_name::<N>();
        debug!("Plus::<{type_name}>::parse() started");

        let mut advanced = 0;
        let mut parse_group = Vec::new();
        let mut ast_group = Vec::new();

        // Match as many nodes as possible before failing
        loop {
            let result = N::parse(token_stream, context);
            match result.1 {
                Ok(n) => {
                    parse_group.push(n.parse_node);
                    ast_group.push(n.ast_node);
                }
                Err(e) => {
                    // Ignore the actual error if the next token was not matched
                    if result.0 == 1 {
                        token_stream.rev();
                        // Unless no nodes have been matched, then propagate the error anyway
                        if parse_group.is_empty() {
                            trace!(
                                "[Plus::<{type_name}>::parse()] Plus quantifier matched no nodes"
                            );
                            return (advanced + result.0, Err(e));
                        }
                    // Propagate the error if the next token WAS matched
                    } else {
                        trace!(
                            "[Plus::<{type_name}>::parse()] Failed on node match attempt {}",
                            parse_group.len() + 1
                        );
                        return (advanced + result.0, Err(e));
                    }
                    break;
                }
            }
            advanced += result.0;
        }

        trace!(
            "[Plus::<{type_name}>::parse()] Matched {} nodes",
            parse_group.len()
        );
        let mark = if let Some(result) = ast_group.first() {
            result.mark
        } else {
            // The marker obviously doesn't matter if nothing was matched, because there's nothing to mark
            Marker::default()
        };
        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(parse_group),
                MarkedAstNode::new(AstNode::multiple(ast_group), mark),
            )),
        )
    }
}

impl<N: ParseTreeNode> ParseTreeNode for Maybe<N> {
    /// Leaves the token stream at the first unmatched token.
    ///
    /// Results in an error if the next token IS matched, but the entire node isn't. Otherwise, an error
    /// should come from matching the next node in the caller.
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        let type_name = std::any::type_name::<N>();
        debug!("Maybe::<{type_name}>::parse() started");

        let result = N::parse(token_stream, context);
        match result.1 {
            Ok(n) => {
                trace!("[Maybe::parse()] Did match node");
                (
                    result.0,
                    Ok(ParseTokensRes::new(Self(Some(n.parse_node)), n.ast_node)),
                )
            }
            Err(e) => {
                // Ignore the actual error if the next token was not matched
                if result.0 == 1 {
                    token_stream.rev();
                // Propagate the error if the next token WAS matched
                } else {
                    trace!("[Maybe::parse()] Failed node match");
                    return (result.0, Err(e));
                }

                trace!("[Maybe::parse()] Did not match node");
                (
                    0,
                    Ok(ParseTokensRes::new(
                        Self(None),
                        MarkedAstNode::new(AstNode::empty, Marker::default()),
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for ProgramNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ProgramNode::parse() started");
        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            error!("[ProgramNode::parse()] The token stream somehow ended early");
            return (
                1,
                Err(ParseError::general(
                    "Grammar error: the token stream somehow ended early...",
                )),
            );
        };

        let mut advanced = 1;

        match first {
            Token::END => {
                trace!("[ProgramNode::parse()] Started END arm");
                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::None,
                        AstNode::from_program_1(MarkedAstNode::new(
                            AstNode::empty,
                            Marker::default(),
                        )),
                    )),
                )
            }
            _ => {
                trace!("[ProgramNode::parse()] Started Scoped* arm");
                advanced -= 1;
                token_stream.rev();

                /* `Scoped*` */
                let scoped_star =
                    match_meta_node!(ScopedNode, Star, token_stream, context, advanced);

                /* `END` */
                // Error message is based on what a `Scoped` can start with
                match_token!(Token::END, "unexpected indentation", token_stream, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Some(scoped_star.parse_node),
                        AstNode::from_program_2(scoped_star.ast_node),
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for ScopedNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ScopedNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (
                1,
                Err(ParseError::general(
                    "Grammar error: the token stream somehow ended early...",
                )),
            );
        };

        let mut advanced = 1;

        match first {
            Token::NEWLINE(_, _) => {
                trace!("[ScopedNode::parse()] Started NEWLINE arm");
                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::None,
                        AstNode::from_scoped_1(MarkedAstNode::new(
                            AstNode::empty,
                            Marker::default(),
                        )),
                    )),
                )
            }
            Token::INDENT(n, _, _) => {
                trace!(
                    "[ScopedNode::parse()] Started INDENT{{{}}} arm",
                    context.indentation
                );

                /* `Indent{n}` */
                if *n > context.indentation {
                    trace!(
                        "[ScopedNode::parse()] Too many indentations, {} expected",
                        context.indentation
                    );
                    return (
                        advanced,
                        Err(ParseError::marked(
                            &format!("too many indentations, {} expected", context.indentation),
                            first.line_and_col().0,
                            0,
                        )),
                    );
                } else if *n < context.indentation {
                    trace!(
                        "[ScopedNode::parse()] Too few indentations, {} expected",
                        context.indentation
                    );
                    return (
                        advanced,
                        Err(ParseError::marked(
                            &format!("too few indentations, {} expected", context.indentation),
                            first.line_and_col().0,
                            0,
                        )),
                    );
                }

                /* `Unit` */
                let unit = match_node!(UnitNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Some(Box::new(unit.parse_node)),
                        AstNode::from_scoped_2(unit.ast_node),
                    )),
                )
            }
            // IMPORTANT: If any new tokens are added here, audit to see if they should be added to the "unexpected token"
            // error message in `ProgramNode::parse()`.
            _ => {
                let (line, col) = first.line_and_col();

                trace!("[ScopedNode::parse()] Unexpected token {first:?}");
                (
                    advanced,
                    Err(ParseError::marked(
                        "unexpected token, expected: newline, indentation",
                        line,
                        col,
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for UnitNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("UnitNode::parse() started");
        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (
                1,
                Err(ParseError::general(
                    "Grammar error: the token stream somehow ended early...",
                )),
            );
        };

        let mut advanced = 1;

        match first {
            Token::KEYWORD(Keyword::If, _, _) => {
                trace!("[UnitNode::parse()] Started KEYWORD(If) arm");

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `MISC(':')` */
                match_token!(
                    Token::MISC(':', _, _),
                    "expected `:`",
                    token_stream,
                    advanced
                );

                /* `Result` */
                let result = match_node!(ResultNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::If(Box::new(expr.parse_node), Box::new(result.parse_node)),
                        AstNode::from_unit_1(expr.ast_node, result.ast_node),
                    )),
                )
            }
            Token::KEYWORD(Keyword::While, _, _) => {
                trace!("[UnitNode::parse()] Started KEYWORD(While) arm");

                let mut context = context.clone();
                context.in_loop = true;
                let context = &context;

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `MISC(':')` */
                match_token!(
                    Token::MISC(':', _, _),
                    "expected `:`",
                    token_stream,
                    advanced
                );

                /* `Result` */
                let result = match_node!(ResultNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::While(Box::new(expr.parse_node), Box::new(result.parse_node)),
                        AstNode::from_unit_2(expr.ast_node, result.ast_node),
                    )),
                )
            }
            Token::KEYWORD(Keyword::For, _, _) => {
                trace!("[UnitNode::parse()] Started KEYWORD(For) arm");

                let mut context = context.clone();
                context.in_loop = true;
                let context = &context;

                /* `NAME` */
                let name = match_token!(
                    Token::NAME(_, _, _),
                    NameTokenNode,
                    "expected a name",
                    token_stream,
                    advanced
                );
                let name_ast = name.as_ast();

                /* `OP(In)` */
                match_token!(
                    Token::OP(Op::In, _, _),
                    "expected `in`",
                    token_stream,
                    advanced
                );

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `MISC(':')` */
                match_token!(
                    Token::MISC(':', _, _),
                    "expected `:`",
                    token_stream,
                    advanced
                );

                /* `Result` */
                let result = match_node!(ResultNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::For(name, Box::new(expr.parse_node), Box::new(result.parse_node)),
                        AstNode::from_unit_3(name_ast, expr.ast_node, result.ast_node),
                    )),
                )
            }
            Token::KEYWORD(Keyword::Continue, row, col) if context.in_loop => {
                trace!("[UnitNode::parse()] Started KEYWORD(Continue) arm");

                /* `NEWLINE` */
                match_token!(
                    Token::NEWLINE(_, _),
                    "expected a newline",
                    token_stream,
                    advanced
                );

                let mark = Marker {
                    row: *row,
                    col: *col,
                };
                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Continue,
                        AstNode::from_unit_4(MarkedAstNode::new(AstNode::r#continue, mark)),
                    )),
                )
            }
            Token::KEYWORD(Keyword::Break, row, col) if context.in_loop => {
                trace!("[UnitNode::parse()] Started KEYWORD(Break) arm");

                /* `NEWLINE` */
                match_token!(
                    Token::NEWLINE(_, _),
                    "expected a newline",
                    token_stream,
                    advanced
                );

                let mark = Marker {
                    row: *row,
                    col: *col,
                };
                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Break,
                        AstNode::from_unit_5(MarkedAstNode::new(AstNode::r#break, mark)),
                    )),
                )
            }
            Token::KEYWORD(Keyword::Return, _, _) if context.in_function => {
                trace!("[UnitNode::parse()] Started KEYWORD(Return) arm");

                /* `Expr?` */
                let expr_maybe = match_meta_node!(ExprNode, Maybe, token_stream, context, advanced);

                /* `NEWLINE` */
                match_token!(
                    Token::NEWLINE(_, _),
                    "expected a newline",
                    token_stream,
                    advanced
                );

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Return(expr_maybe.parse_node),
                        AstNode::from_unit_6(expr_maybe.ast_node),
                    )),
                )
            }
            Token::KEYWORD(Keyword::Def, _, _) => {
                trace!("[UnitNode::parse()] Started KEYWORD(Def) arm");

                let mut context = context.clone();
                context.in_function = true;
                let context = &context;

                /* `NAME` */
                let name = match_token!(
                    Token::NAME(_, _, _),
                    NameTokenNode,
                    "expected a name",
                    token_stream,
                    advanced
                );
                let name_ast = name.as_ast();

                /* `BRACKET('(')` */
                match_token!(
                    Token::BRACKET('(', _, _),
                    "expected a `(`",
                    token_stream,
                    advanced
                );

                /* `Params?` */
                let params_maybe =
                    match_meta_node!(ParamsNode, Maybe, token_stream, context, advanced);

                /* `BRACKET(')')` */
                match_token!(
                    Token::BRACKET(')', _, _),
                    "expected a `)`",
                    token_stream,
                    advanced
                );

                /* `MISC(':')` */
                match_token!(
                    Token::MISC(':', _, _),
                    "expected a `:`",
                    token_stream,
                    advanced
                );

                /* `Body` */
                let body = match_node!(BodyNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Def(name, params_maybe.parse_node, Box::new(body.parse_node)),
                        AstNode::from_unit_7(name_ast, params_maybe.ast_node, body.ast_node),
                    )),
                )
            }
            Token::NAME(_, _, _) => {
                trace!("[UnitNode::parse()] Started NAME arm");

                /* `NAME` */
                let name = NameTokenNode::from_token(first);
                let name_ast = name.as_ast();

                /* `SideEffect` */
                let side_effect = match_node!(SideEffectNode, token_stream, context, advanced);

                /* `NEWLINE` */
                match_token!(
                    Token::NEWLINE(_, _),
                    "expected a newline",
                    token_stream,
                    advanced
                );

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Name(name, Box::new(side_effect.parse_node)),
                        AstNode::from_unit_8(name_ast, side_effect.ast_node),
                    )),
                )
            }
            _ => {
                let (line, col) = first.line_and_col();

                trace!("[UnitNode::parse()] Unexpected token {first:?}");
                (
                    advanced,
                    Err(ParseError::marked(
                        "unexpected token, expected: `if`, `while`, `for`, `continue`, `break`, `def`, name",
                        line,
                        col,
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for ResultNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ResultNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (
                1,
                Err(ParseError::general(
                    "Grammar error: the token stream somehow ended early...",
                )),
            );
        };

        let mut advanced = 1;

        match first {
            Token::NEWLINE(_, _) => {
                trace!("[ResultNode::parse()] Started NEWLINE arm");

                let mut context = context.clone();
                context.indentation += 1;
                let context = &context;

                /* `Scoped+` */
                let scoped_plus =
                    match_meta_node!(ScopedNode, Plus, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::MultiLine(scoped_plus.parse_node),
                        AstNode::from_result_1(scoped_plus.ast_node),
                    )),
                )
            }
            Token::NAME(_, _, _) => {
                trace!("[ResultNode::parse()] Started NAME arm");

                /* `NAME` */
                let name = NameTokenNode::from_token(first);
                let name_ast = name.as_ast();

                /* `SideEffect` */
                let side_effect = match_node!(SideEffectNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::InLine(name, Box::new(side_effect.parse_node)),
                        AstNode::from_result_2(name_ast, side_effect.ast_node),
                    )),
                )
            }
            _ => {
                let (line, col) = first.line_and_col();

                (
                    advanced,
                    Err(ParseError::marked(
                        "unexpected token, expected: newline, name",
                        line,
                        col,
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for BodyNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("BodyNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (
                1,
                Err(ParseError::general(
                    "Grammar error: the token stream somehow ended early...",
                )),
            );
        };

        let mut advanced = 1;

        match first {
            Token::NEWLINE(_, _) => {
                trace!("[BodyNode::parse()] Started NEWLINE arm");

                let mut context = context.clone();
                context.indentation += 1;
                let context = &context;

                /* `Scoped+` */
                let scoped_plus =
                    match_meta_node!(ScopedNode, Plus, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::MultiLine(scoped_plus.parse_node),
                        AstNode::from_body_1(scoped_plus.ast_node),
                    )),
                )
            }
            Token::KEYWORD(Keyword::Return, _, _) => {
                trace!("[BodyNode::parse()] Started KEYWORD(Return) arm");

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `NEWLINE` */
                match_token!(
                    Token::NEWLINE(_, _),
                    "expected a newline",
                    token_stream,
                    advanced
                );

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::InLine(Box::new(expr.parse_node)),
                        AstNode::from_body_2(expr.ast_node),
                    )),
                )
            }
            _ => {
                let (line, col) = first.line_and_col();

                (
                    advanced,
                    Err(ParseError::marked(
                        "unexpected token, expected: newline, `return`",
                        line,
                        col,
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for SideEffectNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("SideEffectNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (
                1,
                Err(ParseError::general(
                    "Grammar error: the token stream somehow ended early...",
                )),
            );
        };

        let mut advanced = 1;

        match first {
            Token::BRACKET('(', _, _) => {
                trace!("[SideEffectNode::parse()] Started BRACKET('(') arm");

                /* `List?` */
                let list_maybe = match_meta_node!(ListNode, Maybe, token_stream, context, advanced);

                /* `BRACKET(')')` */
                match_token!(
                    Token::BRACKET(')', _, _),
                    "expected a `)`",
                    token_stream,
                    advanced
                );

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Call(list_maybe.parse_node),
                        AstNode::from_side_effect_1(list_maybe.ast_node),
                    )),
                )
            }
            _ => {
                trace!("[SideEffectNode::parse()] Started Index* arm");

                advanced -= 1;
                token_stream.rev();

                /* `Index*` */
                let index_star = match_meta_node!(IndexNode, Star, token_stream, context, advanced);

                /* `ASOP` */
                let asop = match_token!(
                    Token::ASOP(_, _, _),
                    AsopTokenNode,
                    "expected an assignment operator",
                    token_stream,
                    advanced
                );
                let asop_ast = asop.as_ast();

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Asop(index_star.parse_node, asop, Box::new(expr.parse_node)),
                        AstNode::from_side_effect_2(index_star.ast_node, asop_ast, expr.ast_node),
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for ExprNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ExprNode::parse() started");

        let mut advanced = 0;

        /* `ExprUnary` */
        let expr_unary = match_node!(ExprUnaryNode, token_stream, context, advanced);

        /* `ExprBinary*` */
        let expr_binary_star =
            match_meta_node!(ExprBinaryNode, Star, token_stream, context, advanced);

        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(Box::new(expr_unary.parse_node), expr_binary_star.parse_node),
                AstNode::from_expr(expr_unary.ast_node, expr_binary_star.ast_node),
            )),
        )
    }
}

impl ParseTreeNode for ExprUnaryNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ExprUnaryNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (
                1,
                Err(ParseError::general(
                    "Grammar error: the token stream somehow ended early...",
                )),
            );
        };

        let mut advanced = 1;

        match first {
            Token::OP(Op::Minus, _, _) => {
                trace!("[ExprUnaryNode::parse()] Started OP(Minus) arm");

                /* `ExprUnit` */
                let expr_unary = match_node!(ExprUnitNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Minus(Box::new(expr_unary.parse_node)),
                        AstNode::from_expr_unary_1(expr_unary.ast_node),
                    )),
                )
            }
            Token::OP(Op::Not, _, _) => {
                trace!("[ExprUnaryNode::parse()] Started OP(Not) arm");

                /* `ExprUnit` */
                let expr_unary = match_node!(ExprUnitNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Not(Box::new(expr_unary.parse_node)),
                        AstNode::from_expr_unary_2(expr_unary.ast_node),
                    )),
                )
            }
            _ => {
                trace!("[ExprUnaryNode::parse()] Started ExprUnit arm");

                advanced -= 1;
                token_stream.rev();

                /* `ExprUnit` */
                let expr_unary = match_node!(ExprUnitNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Unit(Box::new(expr_unary.parse_node)),
                        AstNode::from_expr_unary_3(expr_unary.ast_node),
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for ExprUnitNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ExprUnitNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (
                1,
                Err(ParseError::general(
                    "Grammar error: the token stream somehow ended early...",
                )),
            );
        };

        let mut advanced = 1;

        match first {
            Token::NAME(_, _, _) => {
                trace!("[ExprUnitNode::parse()] Started NAME arm");

                /* `NAME` */
                let name = NameTokenNode::from_token(first);
                let name_ast = name.as_ast();

                /* `NameExpr` */
                let name_expr = match_node!(NameExprNode, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Name(name, Box::new(name_expr.parse_node)),
                        AstNode::from_expr_unit_1(name_ast, name_expr.ast_node),
                    )),
                )
            }
            Token::BRACKET('(', _, _) => {
                trace!("[ExprUnitNode::parse()] Started BRACKET('(') arm");

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `BRACKET(')')` */
                match_token!(
                    Token::BRACKET(')', _, _),
                    "expected a `)`",
                    token_stream,
                    advanced
                );

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Paren(Box::new(expr.parse_node)),
                        AstNode::from_expr_unit_2(expr.ast_node),
                    )),
                )
            }
            Token::BRACKET('[', _, _) => {
                trace!("[ExprUnitNode::parse()] Started BRACKET('[') arm");

                /* `List?` */
                let list_maybe = match_meta_node!(ListNode, Maybe, token_stream, context, advanced);

                /* `BRACKET(']')` */
                match_token!(
                    Token::BRACKET(']', _, _),
                    "expected a `]`",
                    token_stream,
                    advanced
                );

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Bracket(list_maybe.parse_node),
                        AstNode::from_expr_unit_3(list_maybe.ast_node),
                    )),
                )
            }
            Token::BRACKET('{', _, _) => {
                trace!("[ExprUnitNode::parse()] Started BRACKET('{{') arm");

                /* `BracExpr?` */
                let brac_expr_maybe =
                    match_meta_node!(BracExprNode, Maybe, token_stream, context, advanced);

                /* `BRACKET('}')` */
                match_token!(
                    Token::BRACKET('}', _, _),
                    "expected a `}`",
                    token_stream,
                    advanced
                );

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Brace(brac_expr_maybe.parse_node),
                        AstNode::from_expr_unit_4(brac_expr_maybe.ast_node),
                    )),
                )
            }
            Token::STRING(_, _, _) => {
                trace!("[ExprUnitNode::parse()] Started STRING arm");

                let s = StringTokenNode::from_token(first);
                let s_ast = s.as_ast();

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::String(s),
                        AstNode::from_expr_unit_5(s_ast),
                    )),
                )
            }
            Token::NUMBER(_, _, _) => {
                trace!("[ExprUnitNode::parse()] Started NUMBER arm");

                let n = NumberTokenNode::from_token(first);
                let n_ast = n.as_ast();

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Number(n),
                        AstNode::from_expr_unit_6(n_ast),
                    )),
                )
            }
            Token::BOOL(_, _, _) => {
                trace!("[ExprUnitNode::parse()] Started BOOL arm");

                let b = BoolTokenNode::from_token(first);
                let b_ast = b.as_ast();

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Bool(b),
                        AstNode::from_expr_unit_7(b_ast),
                    )),
                )
            }
            _ => {
                trace!("[ExprUnitNode::parse()] Unexpected token {first:?}");
                let (line, col) = first.line_and_col();

                (
                    advanced,
                    Err(ParseError::marked(
                        "unexpected token, expected: name, `(`, `[`, `{`, string, number, boolean",
                        line,
                        col,
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for ExprBinaryNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ExprBinaryNode::parse() started");

        let mut advanced = 0;

        /* `OP` */
        let op = match_token!(
            Token::OP(_, _, _),
            OpTokenNode,
            "expected a binary operator",
            token_stream,
            advanced
        );
        if let Op::Not | Op::BWNot = op.0 {
            return (
                advanced,
                Err(ParseError::marked(
                    "unary operator not allowed here",
                    op.1,
                    op.2,
                )),
            );
        }
        let op_ast = op.as_ast();

        /* `ExprUnit` */
        let expr_unit = match_node!(ExprUnitNode, token_stream, context, advanced);

        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(op, Box::new(expr_unit.parse_node)),
                AstNode::from_expr_binary(op_ast, expr_unit.ast_node),
            )),
        )
    }
}

impl ParseTreeNode for NameExprNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("NameExprNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (
                1,
                Err(ParseError::general(
                    "Grammar error: the token stream somehow ended early...",
                )),
            );
        };

        let mut advanced = 1;

        match first {
            Token::BRACKET('(', _, _) => {
                trace!("[NameExprNode::parse()] Started BRACKET('(') arm");

                /* `List?` */
                let list_maybe = match_meta_node!(ListNode, Maybe, token_stream, context, advanced);

                /* `BRACKET(')')` */
                match_token!(
                    Token::BRACKET(')', _, _),
                    "expected a `)`",
                    token_stream,
                    advanced
                );

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Call(list_maybe.parse_node),
                        AstNode::from_name_expr_1(list_maybe.ast_node),
                    )),
                )
            }
            _ => {
                trace!("[NameExprNode::parse()] Started Index* arm");

                advanced -= 1;
                token_stream.rev();

                /* `Index*` */
                let index_star = match_meta_node!(IndexNode, Star, token_stream, context, advanced);

                (
                    advanced,
                    Ok(ParseTokensRes::new(
                        Self::Index(index_star.parse_node),
                        AstNode::from_name_expr_2(index_star.ast_node),
                    )),
                )
            }
        }
    }
}

impl ParseTreeNode for BracExprNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("BracExprNode::parse() started");

        /* `Dict` */
        let dict_result = (|| {
            let mut advanced = 0;
            let dict = match_node!(DictNode, token_stream, context, advanced);

            (
                advanced,
                Ok(ParseTokensRes::new(
                    Self::Dict(Box::new(dict.parse_node)),
                    AstNode::from_brac_expr_1(dict.ast_node),
                )),
            )
        })();

        // If we didn't match a colon (which would have to be the second token), we'll try matching a list instead of a dict.
        if dict_result.1.is_err() && dict_result.0 <= 1 {
            token_stream.rev_nth(dict_result.0);
            let mut advanced = 0;

            /* `List` */
            let list = match_node!(ListNode, token_stream, context, advanced);

            (
                advanced,
                Ok(ParseTokensRes::new(
                    Self::List(Box::new(list.parse_node)),
                    AstNode::from_brac_expr_2(list.ast_node),
                )),
            )
        } else {
            dict_result
        }
    }
}

impl ParseTreeNode for ListNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ListNode::parse() started");

        let mut advanced = 0;

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        /* `ListTail*` */
        let list_tail_star = match_meta_node!(ListTailNode, Star, token_stream, context, advanced);

        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(Box::new(expr.parse_node), list_tail_star.parse_node),
                AstNode::from_list(expr.ast_node, list_tail_star.ast_node),
            )),
        )
    }
}

impl ParseTreeNode for ListTailNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ListTailNode::parse() started");

        let mut advanced = 0;

        /* `MISC(',')` */
        match_token!(
            Token::MISC(',', _, _),
            "expected a `,`",
            token_stream,
            advanced
        );

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(Box::new(expr.parse_node)),
                AstNode::from_list_tail(expr.ast_node),
            )),
        )
    }
}

impl ParseTreeNode for ParamsNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ParamsNode::parse() started");

        let mut advanced = 0;

        /* `NAME` */
        let name = match_token!(
            Token::NAME(_, _, _),
            NameTokenNode,
            "expected a name",
            token_stream,
            advanced
        );
        let name_ast = name.as_ast();

        /* `ParamsTail*` */
        let params_tail_star =
            match_meta_node!(ParamsTailNode, Star, token_stream, context, advanced);

        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(name, params_tail_star.parse_node),
                AstNode::from_params(name_ast, params_tail_star.ast_node),
            )),
        )
    }
}

impl ParseTreeNode for ParamsTailNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        _context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("ParamsTailNode::parse() started");

        let mut advanced = 0;

        /* `MISC(',')` */
        match_token!(
            Token::MISC(',', _, _),
            "expected a `,`",
            token_stream,
            advanced
        );

        /* `NAME` */
        let name = match_token!(
            Token::NAME(_, _, _),
            NameTokenNode,
            "expected a name",
            token_stream,
            advanced
        );
        let name_ast = name.as_ast();

        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(name),
                AstNode::from_params_tail(name_ast),
            )),
        )
    }
}

impl ParseTreeNode for DictNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("DictNode::parse() started");

        let mut advanced = 0;

        /* `STRING` */
        let string = match_token!(
            Token::STRING(_, _, _),
            StringTokenNode,
            "expected a string",
            token_stream,
            advanced
        );
        let string_ast = string.as_ast();

        /* `MISC(':')` */
        match_token!(
            Token::MISC(':', _, _),
            "expected a `:`",
            token_stream,
            advanced
        );

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        /* `DictTail*` */
        let dict_tail_star = match_meta_node!(DictTailNode, Star, token_stream, context, advanced);

        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(string, Box::new(expr.parse_node), dict_tail_star.parse_node),
                AstNode::from_dict(string_ast, expr.ast_node, dict_tail_star.ast_node),
            )),
        )
    }
}

impl ParseTreeNode for DictTailNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("DictTailNode::parse() started");

        let mut advanced = 0;

        /* `MISC(',')` */
        match_token!(
            Token::MISC(',', _, _),
            "expected a `,`",
            token_stream,
            advanced
        );

        /* `STRING` */
        let string = match_token!(
            Token::STRING(_, _, _),
            StringTokenNode,
            "expected a string",
            token_stream,
            advanced
        );
        let string_ast = string.as_ast();

        /* `MISC(':')` */
        match_token!(
            Token::MISC(':', _, _),
            "expected a `:`",
            token_stream,
            advanced
        );

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(string, Box::new(expr.parse_node)),
                AstNode::from_dict_tail(string_ast, expr.ast_node),
            )),
        )
    }
}

impl ParseTreeNode for IndexNode {
    fn parse<'a>(
        token_stream: &mut TwoWayIterator<Token>,
        context: &Context,
    ) -> (usize, Result<ParseTokensRes<Self>, ParseError>) {
        debug!("IndexNode::parse() started");

        let mut advanced = 0;

        /* `BRACKET('[')` */
        match_token!(
            Token::BRACKET('[', _, _),
            "expected a `[`",
            token_stream,
            advanced
        );

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        /* `BRACKET(']')` */
        match_token!(
            Token::BRACKET(']', _, _),
            "expected a `]`",
            token_stream,
            advanced
        );

        (
            advanced,
            Ok(ParseTokensRes::new(
                Self(Box::new(expr.parse_node)),
                AstNode::from_index_node(expr.ast_node),
            )),
        )
    }
}

/* TPG ENDS HERE */
