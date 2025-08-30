#![allow(dead_code)]

use std::fmt::Debug;
use log::{error, debug, trace};

use super::{building_blocks::*, ParseError};
use crate::util::two_way_iterator::TwoWayIterator;

#[derive(Debug, Default, Clone)]
pub struct Context {
    pub indentation: usize,
    pub in_loop: bool,
    pub in_function: bool
}

#[derive(Debug)]
pub struct Star<N: ParseTreeNode> (Vec<Box<N>>);
#[derive(Debug)]
pub struct Plus<N: ParseTreeNode> (Vec<Box<N>>);
#[derive(Debug)]
pub struct Maybe<N: ParseTreeNode> (Option<Box<N>>);

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
    fn parse(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>);
}

#[derive(Debug)]
pub struct OpTokenNode (Op, usize, usize);
impl OpTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::OP(op, line, col) => Self (
                op.clone(), *line, *col
            ),
            t => panic!("Attempted to make `OpTokenNode` from {t:?}")
        }
    }
}

#[derive(Debug)]
pub struct AsopTokenNode (Asop, usize, usize);
impl AsopTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::ASOP(asop, line, col) => Self (
                asop.clone(), *line, *col
            ),
            t => panic!("Attempted to make `AsopTokenNode` from {t:?}")
        }
    }
}

#[derive(Debug)]
pub struct KeywordTokenNode (Keyword, usize, usize);
impl KeywordTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::KEYWORD(kw, line, col) => Self (
                kw.clone(), *line, *col
            ),
            t => panic!("Attempted to make `KeywordTokenNode` from {t:?}")
        }
    }
}

#[derive(Debug)]
pub struct NameTokenNode (String, usize, usize);
impl NameTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::NAME(name, line, col) => Self (
                name.clone(), *line, *col
            ),
            t => panic!("Attempted to make `NameTokenNode` from {t:?}")
        }
    }
}

#[derive(Debug)]
pub struct StringTokenNode (String, usize, usize);
impl StringTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::STRING(s, line, col) => Self (
                s.clone(), *line, *col
            ),
            t => panic!("Attempted to make `StringTokenNode` from {t:?}")
        }
    }
}

#[derive(Debug)]
pub struct NumberTokenNode (f64, usize, usize);
impl NumberTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::NUMBER(n, line, col) => Self (
                *n, *line, *col
            ),
            t => panic!("Attempted to make `NumberTokenNode` from {t:?}")
        }
    }
}

#[derive(Debug)]
pub struct BoolTokenNode (bool, usize, usize);
impl BoolTokenNode {
    pub fn from_token(token: &Token) -> Self {
        match token {
            Token::BOOL(b, line, col) => Self (
                *b, *line, *col
            ),
            t => panic!("Attempted to make `BoolTokenNode` from {t:?}")
        }
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
    Some(Star<ScopedNode>)
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
    Some(Box<UnitNode>)
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
    Name(NameTokenNode, Box<SideEffectNode>)
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
    InLine(NameTokenNode, Box<SideEffectNode>)
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
    InLine(Box<ExprNode>)
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
    Asop(Star<IndexNode>, AsopTokenNode, Box<ExprNode>)
}

/// Any expression that can return a value.
/// 
/// ```
/// Expr: ExprUnary ExprBinary*
/// ```
#[derive(Debug)]
pub struct ExprNode (Box<ExprUnaryNode>, Star<ExprBinaryNode>);

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
    Unit(Box<ExprUnitNode>)
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
    Bool(BoolTokenNode)
}

/// Helper node for Expr to have multiple subexpressions joined through binary operations.
/// 
/// ```
/// ExprBinary: OP ExprUnit
/// ```
#[derive(Debug)]
pub struct ExprBinaryNode (OpTokenNode, Box<ExprUnitNode>);

/// Helper node for ExprUnit to access a NAME in ways outside of basic value-retrieval.
/// 
/// ```
/// NameExpr: BRACKET('(') List? BRACKET(')')
///         | Index*
/// ```
#[derive(Debug)]
pub enum NameExprNode {
    Call(Maybe<ListNode>),
    Index(Star<IndexNode>)
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
    List(Box<ListNode>)
}

/// A comma-separated list of expressions.
/// 
/// ```
/// List: Expr ListTail*
/// ```
#[derive(Debug)]
pub struct ListNode (Box<ExprNode>, Star<ListTailNode>);

/// Helper node for List to have multiple values.
/// 
/// ```
/// ListTail: MISC(',') Expr
/// ```
#[derive(Debug)]
pub struct ListTailNode (Box<ExprNode>);

/// List but only allowing identifiers.
/// 
/// ```
/// Params: NAME ParamsTail*
/// ```
#[derive(Debug)]
pub struct ParamsNode (NameTokenNode, Star<ParamsTailNode>);

/// Helper node for Params to have multiple values.
/// 
/// ```
/// ParamsTail: MISC(',') NAME
/// ```
#[derive(Debug)]
pub struct ParamsTailNode (NameTokenNode);

/// A comma-separated list of key-value pairs.
/// 
/// ```
/// Dict: STRING MISC(':') Expr DictTail*
/// ```
#[derive(Debug)]
pub struct DictNode (StringTokenNode, Box<ExprNode>, Star<DictTailNode>);

/// Helper node for Dict to have multiple key-value pairs.
/// 
/// ```
/// DictTail: MISC(',') STRING MISC(':') Expr
/// ```
#[derive(Debug)]
pub struct DictTailNode (StringTokenNode, Box<ExprNode>);

/// The index of an indexable NAME.
/// 
/// ```
/// Index: BRACKET('[') Expr BRACKET(']')
/// ```
#[derive(Debug)]
pub struct IndexNode (Box<ExprNode>);

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
                return ($advanced, Err(e))
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
            Some(t @ $token_pat) => {
                $token_node::from_token(t)
            },
            Some(t) => {
                trace!("[{}::parse()] {} ({t:?})", stringify!($token_node), $err_message);
                let (line, col) = t.line_and_col();
                return ($advanced, Err(ParseError::marked(
                    $err_message,
                    line, 
                    col
                )));
            },
            None => {
                error!("[{}::parse()] The token stream somehow ended early", stringify!($token_node));
                return ($advanced, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
            }
        }
    }};

    ($token_pat:pat, $err_message:literal, $token_stream:ident, $advanced:ident) => {{
        $advanced += 1;
        match $token_stream.next() {
            Some($token_pat) => {},
            Some(t) => {
                trace!("{} ({t:?} != {})", $err_message, stringify!($token_pat));
                let (line, col) = t.line_and_col();
                return ($advanced, Err(ParseError::marked(
                    $err_message,
                    line, 
                    col
                )));
            },
            None => {
                error!("The token stream somehow ended early");
                return ($advanced, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
            }
        }
    }};
}

pub fn parse_tokens(token_stream: &Vec<Token>) -> Result<ProgramNode, ParseError> {
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
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        let type_name = std::any::type_name::<N>();
        debug!("Star::<{type_name}>::parse() started");

        let mut advanced = 0;
        let mut group = Vec::new();

        // Match as many `N`s as possible before failing
        loop {
            let result = N::parse(token_stream, context);
            match result.1 {
                Ok(n) => group.push(Box::new(n)),
                Err(e) => {
                    // Ignore the actual error if the next token was not matched
                    if result.0 == 1 {
                        token_stream.rev();
                    // Propagate the error if the next token WAS matched
                    } else {
                        trace!("[Star::<{type_name}>::parse()] Failed on node match attempt {}", group.len() + 1);
                        return (advanced + result.0, Err(e));
                    }
                    break;
                }
            }
            advanced += result.0;
        }

        trace!("[Star::<{type_name}>::parse()] Matched {} nodes", group.len());
        (advanced, Ok(Self (group)))
    }
}

impl<N: ParseTreeNode> ParseTreeNode for Plus<N> {
    /// Leaves the token stream at the first unmatched token.
    /// 
    /// Results in an error if, in the current node matching attempt, the next token IS matched but the entire node isn't. Otherwise, an error
    /// should come from matching the next node in the caller.
    /// Also, an error is returned if no nodes are matched.
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        let type_name = std::any::type_name::<N>();
        debug!("Plus::<{type_name}>::parse() started");

        let mut advanced = 0;
        let mut group = Vec::new();

        // Match as many nodes as possible before failing
        loop {
            let result = N::parse(token_stream, context);
            match result.1 {
                Ok(n) => group.push(Box::new(n)),
                Err(e) => {
                    // Ignore the actual error if the next token was not matched
                    if result.0 == 1 {
                        token_stream.rev();
                        // Unless no nodes have been matched, then propagate the error anyway
                        if group.is_empty() {
                            trace!("[Plus::<{type_name}>::parse()] Plus quantifier matched no nodes");
                            return (advanced + result.0, Err(e));
                        }
                    // Propagate the error if the next token WAS matched
                    } else {
                        trace!("[Plus::<{type_name}>::parse()] Failed on node match attempt {}", group.len() + 1);
                        return (advanced + result.0, Err(e));
                    }
                    break;
                }
            }
            advanced += result.0;
        }

        trace!("[Plus::<{type_name}>::parse()] Matched {} nodes", group.len());
        (advanced, Ok(Self (group)))
    }
}

impl<N: ParseTreeNode> ParseTreeNode for Maybe<N> {
    /// Leaves the token stream at the first unmatched token.
    /// 
    /// Results in an error if the next token IS matched, but the entire node isn't. Otherwise, an error
    /// should come from matching the next node in the caller.
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("Maybe::parse() started");

        let result = N::parse(token_stream, context);
        match result.1 {
            Ok(n) => {
                trace!("[Maybe::parse()] Did match node");
                (result.0, Ok(
                    Self ( Some(Box::new(n)) )
                ))
            },
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
                (0, Ok(Self (None)))
            }
        }
    }
}

impl ParseTreeNode for ProgramNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ProgramNode::parse() started");
        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            error!("[ProgramNode::parse()] The token stream somehow ended early");
            return (1, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
        };

        let mut advanced = 1;

        match first {
            Token::END => {
                trace!("[ProgramNode::parse()] Started END arm");
                (advanced, Ok(Self::None))
            },
            _ => {
                trace!("[ProgramNode::parse()] Started Scoped* arm");
                advanced -= 1;
                token_stream.rev();

                /* `Scoped*` */
                let scoped_star = match_meta_node!(ScopedNode, Star, token_stream, context, advanced);

                /* `END` */
                match_token!(Token::END, "unexpected token", token_stream, advanced);

                (advanced, Ok(Self::Some(scoped_star)))
            }
        }
    }
}

impl ParseTreeNode for ScopedNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ScopedNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (1, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
        };

        let mut advanced = 1;

        match first {
            Token::NEWLINE(_, _) => {
                trace!("[ScopedNode::parse()] Started NEWLINE arm");
                (advanced, Ok(Self::None))
            },
            Token::INDENT(n, _, _) => {
                trace!("[ScopedNode::parse()] Started INDENT{{{}}} arm", context.indentation);

                /* `Indent{n}` */
                if *n > context.indentation {
                    trace!("[ScopedNode::parse()] Too many indentations, {} expected", context.indentation);
                    return (advanced, Err(ParseError::marked(
                        &format!("too many indentations, {} expected", context.indentation), 
                        first.line_and_col().0,
                        0
                    )));
                } else if *n < context.indentation {
                    trace!("[ScopedNode::parse()] Too few indentations, {} expected", context.indentation);
                    return (advanced, Err(ParseError::marked(
                        &format!("too few indentations, {} expected", context.indentation), 
                        first.line_and_col().0,
                        0
                    )));
                }

                /* `Unit` */
                let unit = match_node!(UnitNode, token_stream, context, advanced);

                (advanced, Ok(Self::Some(Box::new(unit))))
            },
            _ => {
                let (line, col) = first.line_and_col();

                trace!("[ScopedNode::parse()] Unexpected token {first:?}");
                (advanced, Err(ParseError::marked(
                    "unexpected token",
                    line,
                    col
                )))
            }
        }
    }
}

impl ParseTreeNode for UnitNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("UnitNode::parse() started");
        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (1, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
        };

        let mut advanced = 1;

        match first {
            Token::KEYWORD(Keyword::If, _, _) => {
                trace!("[UnitNode::parse()] Started KEYWORD(If) arm");

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `MISC(':')` */
                match_token!(Token::MISC(':', _, _), "expected `:`", token_stream, advanced);

                /* `Result` */
                let result = match_node!(ResultNode, token_stream, context, advanced);

                (advanced, Ok(Self::If(Box::new(expr), Box::new(result))))
            },
            Token::KEYWORD(Keyword::While, _, _) => {
                trace!("[UnitNode::parse()] Started KEYWORD(While) arm");

                let mut context = context.clone();
                context.in_loop = true;
                let context = &context;

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `MISC(':')` */
                match_token!(Token::MISC(':', _, _), "expected `:`", token_stream, advanced);

                /* `Result` */
                let result = match_node!(ResultNode, token_stream, context, advanced);

                (advanced, Ok(Self::While(Box::new(expr), Box::new(result))))
            },
            Token::KEYWORD(Keyword::For, _, _) => {
                trace!("[UnitNode::parse()] Started KEYWORD(For) arm");

                let mut context = context.clone();
                context.in_loop = true;
                let context = &context;

                /* `NAME` */
                let name = match_token!(Token::NAME(_, _, _), NameTokenNode, "expected a name", token_stream, advanced);

                /* `OP(In)` */
                match_token!(Token::OP(Op::In, _, _), "expected `in`", token_stream, advanced);

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `MISC(':')` */
                match_token!(Token::MISC(':', _, _), "expected `:`", token_stream, advanced);

                /* `Result` */
                let result = match_node!(ResultNode, token_stream, context, advanced);

                (advanced, Ok(Self::For(name, Box::new(expr), Box::new(result))))
            },
            Token::KEYWORD(Keyword::Continue, _, _) if context.in_loop => {
                trace!("[UnitNode::parse()] Started KEYWORD(Continue) arm");

                /* `NEWLINE` */
                match_token!(Token::NEWLINE(_, _), "expected a newline", token_stream, advanced);

                (advanced, Ok(Self::Continue))
            },
            Token::KEYWORD(Keyword::Break, _, _) if context.in_loop => {
                trace!("[UnitNode::parse()] Started KEYWORD(Break) arm");

                /* `NEWLINE` */
                match_token!(Token::NEWLINE(_, _), "expected a newline", token_stream, advanced);

                (advanced, Ok(Self::Break))
            },
            Token::KEYWORD(Keyword::Return, _, _) if context.in_function => {
                trace!("[UnitNode::parse()] Started KEYWORD(Return) arm");

                /* `Expr?` */
                let expr_maybe = match_meta_node!(ExprNode, Maybe, token_stream, context, advanced);

                /* `NEWLINE` */
                match_token!(Token::NEWLINE(_, _), "expected a newline", token_stream, advanced);

                (advanced, Ok(Self::Return(expr_maybe)))
            },
            Token::KEYWORD(Keyword::Def, _, _) => {
                trace!("[UnitNode::parse()] Started KEYWORD(Def) arm");

                let mut context = context.clone();
                context.in_function = true;
                let context = &context;

                /* `NAME` */
                let name = match_token!(Token::NAME(_, _, _), NameTokenNode, "expected a name", token_stream, advanced);

                /* `BRACKET('(')` */ 
                match_token!(Token::BRACKET('(', _, _), "expected a `(`", token_stream, advanced);

                /* `Params?` */
                let params_maybe = match_meta_node!(ParamsNode, Maybe, token_stream, context, advanced);

                /* `BRACKET(')')` */
                match_token!(Token::BRACKET(')', _, _), "expected a `)`", token_stream, advanced);

                /* `MISC(':')` */
                match_token!(Token::MISC(':', _, _), "expected a `:`", token_stream, advanced);

                /* `Body` */
                let body = match_node!(BodyNode, token_stream, context, advanced);

                (advanced, Ok(Self::Def(name, params_maybe, Box::new(body))))
            },
            Token::NAME(_, _, _) => {
                trace!("[UnitNode::parse()] Started NAME arm");

                /* `NAME` */
                let name = NameTokenNode::from_token(first);

                /* `SideEffect` */
                let side_effect = match_node!(SideEffectNode, token_stream, context, advanced);

                /* `NEWLINE` */
                match_token!(Token::NEWLINE(_, _), "expected a newline", token_stream, advanced);

                (advanced, Ok(Self::Name(name, Box::new(side_effect))))
            },
            _ => {
                let (line, col) = first.line_and_col();

                trace!("[UnitNode::parse()] Unexpected token {first:?}");
                (advanced, Err(ParseError::marked(
                    "unexpected token",
                    line,
                    col
                )))
            }
        }
    }
}

impl ParseTreeNode for ResultNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ResultNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (1, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
        };

        let mut advanced = 1;

        match first {
            Token::NEWLINE(_, _) => {
                trace!("[ResultNode::parse()] Started NEWLINE arm");

                let mut context = context.clone();
                context.indentation += 1;
                let context = &context;

                /* `Scoped+` */
                let scoped_plus = match_meta_node!(ScopedNode, Plus, token_stream, context, advanced);

                (advanced, Ok(Self::MultiLine(scoped_plus)))
            },
            Token::NAME(_, _, _) => {
                trace!("[ResultNode::parse()] Started NAME arm");

                /* `NAME` */
                let name = NameTokenNode::from_token(first);

                /* `SideEffect` */
                let side_effect = match_node!(SideEffectNode, token_stream, context, advanced);

                (advanced, Ok(Self::InLine(name, Box::new(side_effect))))
            },
            _ => {
                let (line, col) = first.line_and_col();

                (advanced, Err(ParseError::marked(
                    "unexpected token",
                    line,
                    col
                )))
            }
        }
    }
}

impl ParseTreeNode for BodyNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("BodyNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (1, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
        };

        let mut advanced = 1;

        match first {
            Token::NEWLINE(_, _) => {
                trace!("[BodyNode::parse()] Started NEWLINE arm");

                let mut context = context.clone();
                context.indentation += 1;
                let context = &context;

                /* `Scoped+` */
                let scoped_plus = match_meta_node!(ScopedNode, Plus, token_stream, context, advanced);

                (advanced, Ok(Self::MultiLine(scoped_plus)))
            },
            Token::KEYWORD(Keyword::Return, _, _) => {
                trace!("[BodyNode::parse()] Started KEYWORD(Return) arm");

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `NEWLINE` */
                match_token!(Token::NEWLINE(_, _), "expected a newline", token_stream, advanced);

                (advanced, Ok(Self::InLine(Box::new(expr))))
            },
            _ => {
                let (line, col) = first.line_and_col();

                (advanced, Err(ParseError::marked(
                    "unexpected token",
                    line,
                    col
                )))
            }
        }
    }
}

impl ParseTreeNode for SideEffectNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("SideEffectNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (1, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
        };

        let mut advanced = 1;

        match first {
            Token::BRACKET('(', _, _) => {
                trace!("[SideEffectNode::parse()] Started BRACKET('(') arm");

                /* `List?` */
                let list_maybe = match_meta_node!(ListNode, Maybe, token_stream, context, advanced);

                /* `BRACKET(')')` */
                match_token!(Token::BRACKET(')', _, _), "expected a `)`", token_stream, advanced);

                (advanced, Ok(Self::Call(list_maybe)))
            },
            _ => {
                trace!("[SideEffectNode::parse()] Started Index* arm");

                advanced -= 1;
                token_stream.rev();

                /* `Index*` */
                let index_star = match_meta_node!(IndexNode, Star, token_stream, context, advanced);

                /* `ASOP` */
                let asop = match_token!(Token::ASOP(_, _, _), AsopTokenNode, "expected an assignment operator", token_stream, advanced);

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                (advanced, Ok(Self::Asop(index_star, asop, Box::new(expr))))
            }
        }
    }
}

impl ParseTreeNode for ExprNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ExprNode::parse() started");
        
        let mut advanced = 0;

        /* `ExprUnary` */
        let expr_unary = match_node!(ExprUnaryNode, token_stream, context, advanced);

        /* `ExprBinary*` */
        let expr_binary_star = match_meta_node!(ExprBinaryNode, Star, token_stream, context, advanced);

        (advanced, Ok(Self (Box::new(expr_unary), expr_binary_star)))
    }
}

impl ParseTreeNode for ExprUnaryNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ExprUnaryNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (1, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
        };

        let mut advanced = 1;

        match first {
            Token::OP(Op::Minus, _, _) => {
                trace!("[ExprUnaryNode::parse()] Started OP(Minus) arm");

                /* `ExprUnit` */
                let expr_unary = match_node!(ExprUnitNode, token_stream, context, advanced);

                (advanced, Ok(Self::Minus(Box::new(expr_unary))))
            },
            Token::OP(Op::Not, _, _) => {
                trace!("[ExprUnaryNode::parse()] Started OP(Not) arm");

                /* `ExprUnit` */
                let expr_unary = match_node!(ExprUnitNode, token_stream, context, advanced);

                (advanced, Ok(Self::Not(Box::new(expr_unary))))
            },
            _ => {
                trace!("[ExprUnaryNode::parse()] Started ExprUnit arm");

                advanced -= 1;
                token_stream.rev();

                /* `ExprUnit` */
                let expr_unary = match_node!(ExprUnitNode, token_stream, context, advanced);

                (advanced, Ok(Self::Unit(Box::new(expr_unary))))
            }
        }
    }
}

impl ParseTreeNode for ExprUnitNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ExprUnitNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (1, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
        };

        let mut advanced = 1;

        match first {
            Token::NAME(_, _, _) => {
                trace!("[ExprUnitNode::parse()] Started NAME arm");

                /* `NAME` */
                let name = NameTokenNode::from_token(first);

                /* `NameExpr` */
                let name_expr = match_node!(NameExprNode, token_stream, context, advanced);

                (advanced, Ok(Self::Name(name, Box::new(name_expr))))
                
            },
            Token::BRACKET('(', _, _) => {
                trace!("[ExprUnitNode::parse()] Started BRACKET('(') arm");

                /* `Expr` */
                let expr = match_node!(ExprNode, token_stream, context, advanced);

                /* `BRACKET(')')` */
                match_token!(Token::BRACKET(')', _, _), "expected a `)`", token_stream, advanced);

                (advanced, Ok(Self::Paren(Box::new(expr))))
            },
            Token::BRACKET('[', _, _) => {
                trace!("[ExprUnitNode::parse()] Started BRACKET('[') arm");

                /* `List?` */
                let list_maybe = match_meta_node!(ListNode, Maybe, token_stream, context, advanced);

                /* `BRACKET(']')` */
                match_token!(Token::BRACKET(']', _, _), "expected a `]`", token_stream, advanced);

                (advanced, Ok(Self::Bracket(list_maybe)))
            },
            Token::BRACKET('{', _, _) => {
                trace!("[ExprUnitNode::parse()] Started BRACKET('{{') arm");

                /* `BracExpr?` */
                let brac_expr_maybe = match_meta_node!(BracExprNode, Maybe, token_stream, context, advanced);

                /* `BRACKET('}')` */
                match_token!(Token::BRACKET('}', _, _), "expected a `}`", token_stream, advanced);

                (advanced, Ok(Self::Brace(brac_expr_maybe)))
            },
            Token::STRING(_, _, _) => {
                trace!("[ExprUnitNode::parse()] Started STRING arm");

                (advanced, Ok(Self::String(StringTokenNode::from_token(first))))
            },
            Token::NUMBER(_, _, _) => {
                trace!("[ExprUnitNode::parse()] Started NUMBER arm");

                (advanced, Ok(Self::Number(NumberTokenNode::from_token(first))))
            },
            Token::BOOL(_, _, _) => {
                trace!("[ExprUnitNode::parse()] Started BOOL arm");

                (advanced, Ok(Self::Bool(BoolTokenNode::from_token(first))))
            },
            _ => {
                trace!("[ExprUnitNode::parse()] Unexpected token {first:?}");
                let (line, col) = first.line_and_col();

                (advanced, Err(ParseError::marked(
                    "unexpected token",
                    line,
                    col
                )))
            }
        }
    }
}

impl ParseTreeNode for ExprBinaryNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ExprBinaryNode::parse() started");

        let mut advanced = 0;

        /* `OP` */
        let op = match_token!(Token::OP(_, _, _), OpTokenNode, "expected a binary operator", token_stream, advanced);
        if let Op::Not | Op::BWNot = op.0 {
            return (advanced, Err(ParseError::marked(
                "unary operator not allowed here",
                op.1,
                op.2
            )))
        }

        /* `ExprUnit` */
        let expr_unit = match_node!(ExprUnitNode, token_stream, context, advanced);

        (advanced, Ok(Self(op, Box::new(expr_unit))))
    }
}

impl ParseTreeNode for NameExprNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("NameExprNode::parse() started");

        let first = if let Some(token) = token_stream.next() {
            token
        } else {
            return (1, Err(ParseError::general("Grammar error: the token stream somehow ended early...")));
        };

        let mut advanced = 1;

        match first {
            Token::BRACKET('(', _, _) => {
                trace!("[NameExprNode::parse()] Started BRACKET('(') arm");

                /* `List?` */
                let list_maybe = match_meta_node!(ListNode, Maybe, token_stream, context, advanced);

                /* `BRACKET(')')` */
                match_token!(Token::BRACKET(')', _, _), "expected a `)`", token_stream, advanced);

                (advanced, Ok(Self::Call(list_maybe)))
            },
            _ => {
                trace!("[NameExprNode::parse()] Started Index* arm");

                advanced -= 1;
                token_stream.rev();

                /* `Index*` */
                let index_star = match_meta_node!(IndexNode, Star, token_stream, context, advanced);

                (advanced, Ok(Self::Index(index_star)))
            }
        }
    }
}

impl ParseTreeNode for BracExprNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("BracExprNode::parse() started");

        /* `Dict` */
        let dict_result = (|| {
            let mut advanced = 0;
            let dict = match_node!(DictNode, token_stream, context, advanced);

            (advanced, Ok(Self::Dict(Box::new(dict))))
        })();

        // If we didn't match a colon (which would have to be the second token), we'll try matching a list instead of a dict.
        if dict_result.1.is_err() && dict_result.0 <= 1 {
            token_stream.rev_nth(dict_result.0);
            let mut advanced = 0;

            /* `List` */
            let list = match_node!(ListNode, token_stream, context, advanced);

            (advanced, Ok(Self::List(Box::new(list))))
        } else {
            dict_result
        }
    }
}

impl ParseTreeNode for ListNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ListNode::parse() started");

        let mut advanced = 0;

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        /* `ListTail*` */
        let list_tail_star = match_meta_node!(ListTailNode, Star, token_stream, context, advanced);

        (advanced, Ok(Self(Box::new(expr), list_tail_star)))
    }
}

impl ParseTreeNode for ListTailNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ListTailNode::parse() started");

        let mut advanced = 0;

        /* `MISC(',')` */
        match_token!(Token::MISC(',', _, _), "expected a `,`", token_stream, advanced);

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        (advanced, Ok(Self(Box::new(expr))))
    }
}

impl ParseTreeNode for ParamsNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ParamsNode::parse() started");

        let mut advanced = 0;

        /* `NAME` */
        let name = match_token!(Token::NAME(_, _, _), NameTokenNode, "expected a name", token_stream, advanced);

        /* `ParamsTail*` */
        let params_tail_star = match_meta_node!(ParamsTailNode, Star, token_stream, context, advanced);

        (advanced, Ok(Self(name, params_tail_star)))
    }
}

impl ParseTreeNode for ParamsTailNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, _context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("ParamsTailNode::parse() started");

        let mut advanced = 0;

        /* `MISC(',')` */
        match_token!(Token::MISC(',', _, _), "expected a `,`", token_stream, advanced);

        /* `NAME` */
        let name = match_token!(Token::NAME(_, _, _), NameTokenNode, "expected a name", token_stream, advanced);

        (advanced, Ok(Self(name)))
    }
}

impl ParseTreeNode for DictNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("DictNode::parse() started");

        let mut advanced = 0;

        /* `STRING` */
        let string = match_token!(Token::STRING(_, _, _), StringTokenNode, "expected a string", token_stream, advanced);

        /* `MISC(':')` */
        match_token!(Token::MISC(':', _, _), "expected a `:`", token_stream, advanced);

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        /* `DictTail*` */
        let dict_tail_star = match_meta_node!(DictTailNode, Star, token_stream, context, advanced);

        (advanced, Ok(Self(string, Box::new(expr), dict_tail_star)))
    }
}

impl ParseTreeNode for DictTailNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("DictTailNode::parse() started");

        let mut advanced = 0;

        /* `MISC(',')` */
        match_token!(Token::MISC(',', _, _), "expected a `,`", token_stream, advanced);

        /* `STRING` */
        let string = match_token!(Token::STRING(_, _, _), StringTokenNode, "expected a string", token_stream, advanced);

        /* `MISC(':')` */
        match_token!(Token::MISC(':', _, _), "expected a `:`", token_stream, advanced);

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        (advanced, Ok(Self(string, Box::new(expr))))
    }
}

impl ParseTreeNode for IndexNode {
    fn parse<'a>(token_stream: &mut TwoWayIterator<Token>, context: &Context) -> (usize, Result<Self, ParseError>) {
        debug!("IndexNode::parse() started");

        let mut advanced = 0;

        /* `BRACKET('[')` */
        match_token!(Token::BRACKET('[', _, _), "expected a `[`", token_stream, advanced);

        /* `Expr` */
        let expr = match_node!(ExprNode, token_stream, context, advanced);

        /* `BRACKET(']')` */
        match_token!(Token::BRACKET(']', _, _), "expected a `]`", token_stream, advanced);

        (advanced, Ok(Self(Box::new(expr))))
    }
}

/* TPG ENDS HERE */
