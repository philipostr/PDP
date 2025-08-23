/// All token variants come equipped with a pair of `usize` values
/// to signify their line and col respectively. This is except for `END`,
/// which is quite obviously at the end of the token stream and does not
/// stand for any real lexeme.
#[derive(Debug, PartialEq)]
pub enum Token {
    INDENT(usize, usize),
    OP(Op, usize, usize),
    ASOP(Asop, usize, usize),
    KEYWORD(Keyword, usize, usize),
    NAME(String, usize, usize),
    BRACKET(char, usize, usize),
    STRING(String, usize, usize),
    NUMBER(f64, usize, usize),
    BOOL(bool, usize, usize),
    NEWLINE(usize, usize),
    MISC(char, usize, usize),
    END,
}

/// All operators are binary only, except for:
/// * `Plus`: Binary AND Unary
/// * `Minus`: Binary AND Unary
/// * `Not`: Unary ONLY
/// * `BWNot`: Unary ONLY
#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    /* Arithmetic operators */
    Plus,   // +
    Minus,  // -
    Mult,   // *
    Div,    // /
    IntDiv, // //
    Mod,    // %
    Exp,    // **

    /* Comparison operators */
    Eq,  // ==
    Neq, // !=
    Gt,  // >
    Gte, // >=
    Lt,  // <
    Lte, // <=

    /* Logical operators */
    And, // and
    Or,  // or
    Not, // not

    /* Bitwise operators */
    BWAnd,   // &
    BWOr,    // |
    BWNot,   // ~
    Xor,     // ^
    ShLeft,  // <<
    ShRight, // >>

    /* Membership operators */
    In,    // in
    NotIn, // not in
}

#[derive(Debug, PartialEq, Eq)]
pub enum Asop {
    Assign,        // =
    AddAssign,     // +=
    SubAssign,     // -=
    MultAssign,    // *=
    DivAssign,     // /=
    ModAssign,     // %=
    IntDivAssign,  // //=
    ExpAssign,     // **=
    BWAndAssign,   // &=
    BWOrAssign,    // |=
    BWNotAssign,   // ~=
    XorAssign,     // ^=
    ShLeftAssign,  // <<=
    ShRightAssign, // >>=
}

#[derive(Debug, PartialEq, Eq)]
pub enum Keyword {
    If,
    While,
    For,
    Continue,
    Break,
    Return,
    Def,
}
