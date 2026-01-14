/// All token variants come equipped with a pair of `usize` values
/// to signify their line and col respectively. This is except for `END`,
/// which is quite obviously at the end of the token stream and does not
/// stand for any real lexeme.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq)]
pub enum Token {
    INDENT(usize, usize, usize),
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

impl Token {
    pub fn line_and_col(&self) -> (usize, usize) {
        use Token::*;
        match self {
            INDENT(_, line, col) => (*line, *col),
            OP(_, line, col) => (*line, *col),
            ASOP(_, line, col) => (*line, *col),
            KEYWORD(_, line, col) => (*line, *col),
            NAME(_, line, col) => (*line, *col),
            BRACKET(_, line, col) => (*line, *col),
            STRING(_, line, col) => (*line, *col),
            NUMBER(_, line, col) => (*line, *col),
            BOOL(_, line, col) => (*line, *col),
            NEWLINE(line, col) => (*line, *col),
            MISC(_, line, col) => (*line, *col),
            END => (0, 0),
        }
    }
}

/// All operators are binary only, except for:
/// * `Plus`: Binary AND Unary
/// * `Minus`: Binary AND Unary
/// * `Not`: Unary ONLY
/// * `BWNot`: Unary ONLY
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /* Internal operators */
    /// Nothing. Created from `Asop::as_op()`
    Identity,
}

impl Op {
    pub fn dunderscore_method(&self) -> &'static str {
        match self {
            Op::Plus => "__add__",
            Op::Minus => "__sub__",
            Op::Mult => "__mul__",
            Op::Div => "__truediv__",
            Op::IntDiv => "__floordiv__",
            Op::Mod => "__mod__",
            Op::Exp => "__pow__",
            Op::Eq => "__eq__",
            Op::Neq => "__ne__",
            Op::Gt => "__gt__",
            Op::Gte => "__ge__",
            Op::Lt => "__lt__",
            Op::Lte => "__le__",
            Op::And => "__and__",
            Op::Or => "__or__",
            Op::Not => "__inv__",
            Op::BWAnd => "__bwand__",
            Op::BWOr => "__bwor__",
            Op::BWNot => "__bwinv__",
            Op::Xor => "__xor__",
            Op::ShLeft => "__lshift__",
            Op::ShRight => "__rshift__",
            Op::In => "__contains__",
            Op::NotIn => "__ncontains__",
            Op::Identity => "",
        }
    }

    pub fn dunderscore_method_unary(&self) -> &'static str {
        match self {
            Op::Minus => "__neg__",
            _ => self.dunderscore_method(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Asop {
    pub fn as_op(&self) -> Op {
        match self {
            Asop::Assign => Op::Identity,
            Asop::AddAssign => Op::Plus,
            Asop::SubAssign => Op::Minus,
            Asop::MultAssign => Op::Mult,
            Asop::DivAssign => Op::Div,
            Asop::ModAssign => Op::Mod,
            Asop::IntDivAssign => Op::IntDiv,
            Asop::ExpAssign => Op::Exp,
            Asop::BWAndAssign => Op::BWAnd,
            Asop::BWOrAssign => Op::BWOr,
            Asop::BWNotAssign => Op::BWNot,
            Asop::XorAssign => Op::Xor,
            Asop::ShLeftAssign => Op::ShLeft,
            Asop::ShRightAssign => Op::ShRight,
        }
    }

    pub fn dunderscore_method(&self) -> &'static str {
        self.as_op().dunderscore_method()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    If,
    While,
    For,
    Continue,
    Break,
    Return,
    Def,
}
