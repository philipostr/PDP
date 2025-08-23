use super::building_blocks::*;

const SYMBOLS: [char; 21] = [
    '+', '-', '*', '/', '%', '!', '>', '<', '&', '|',
    '^', '~', '=', '(', ')', '{', '}', '[', ']', ',',
    ':'
];

/// Convenience trait to allow the many `(&[char]).starts_with(&str)` invocations in
/// `Lexer::identify()`.
trait StartsWithStr {
    fn starts_with_str(&self, needle: &str) -> bool;
}

impl StartsWithStr for &[char] {
    fn starts_with_str(&self, needle: &str) -> bool {
        self.iter()
            .zip(needle.chars())
            .all(|(l, r)| {
                l == &r
            })
    }
}

#[derive(Debug, Default)]
pub struct Lexer {
    finished: bool,
    tokens: Vec<Token>,
    next_start_line: usize,
    next_start_col: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finalize(&mut self) -> core::slice::Iter<Token> {
        // Push an extra newline before the end because the grammar requires it
        self.tokens.push(Token::NEWLINE(self.next_start_line, self.next_start_col));
        self.tokens.push(Token::END);

        self.finished = true;
        self.tokens.iter()
    }

    /// Used to advance a character iterator by lexeme. It identifies the lexeme, appends its lexed `Token` value to
    /// `self.tokens`, and returns how many characters the iterator was advanced by.
    ///
    /// Returns a `Err(String)` if something couldn't be lexed properly.
    pub fn identify(&mut self, line: &[char]) -> Result<usize, String> {
        if self.finished {
            return Err("this lexer has finished its job.".to_string());
        }

        // == Actual tokenization logic starts here == //
        if line.is_empty() { // newline
            self.tokens.push(Token::NEWLINE(self.next_start_line, self.next_start_col));
            self.next_start_col = 0;
            self.next_start_line += 1;
            Ok(1)
        } else if line[0] == ' ' {
            if self.next_start_col == 0 { // Count indentation spaces at the start of a line
                let mut num_spaces = 0;

                // Find how many spaces the line starts with
                for c in line {
                    if *c == ' ' {
                        num_spaces += 1;
                    } else if *c == '#' { // we don't care about indentations if the line is only a comment
                        self.next_start_line += 1;
                        return Ok(line.len() + 1);
                    } else {
                        break;
                    }
                }

                // Make sure the amount of spaces is valid
                if num_spaces % 4 != 0 {
                    return Err("unknown amount of indentations, number of spaces should be a multiple of 4".to_string());
                }

                // Finalize the identification
                for i in 0..(num_spaces / 4) {
                    self.tokens.push(Token::INDENT(self.next_start_line, i*4));
                }
                self.next_start_col += num_spaces;
                Ok(num_spaces)

            } else { // Ignore random spaces inside a line
                let mut num_spaces = 1;

                // Count the spaces
                for c in &line[1..] {
                    if *c == ' ' {
                        num_spaces += 1;
                    } else {
                        break;
                    }
                }

                self.next_start_col += num_spaces;
                Ok(num_spaces)
            }
        } else if line[0] == '#' {
            self.next_start_col = 0;
            self.next_start_line += 1;
            Ok(line.len() + 1) // + 1 because we're pretending like we found a newline
        } else if line.starts_with_str("if") && Self::word_boundary(line, 2) {
            self.tokens.push(Token::KEYWORD(Keyword::If, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("while") && Self::word_boundary(line, 5) {
            self.tokens.push(Token::KEYWORD(Keyword::While, self.next_start_line, self.next_start_col));
            self.next_start_col += 5;
            Ok(5)
        } else if line.starts_with_str("for") && Self::word_boundary(line, 3) {
            self.tokens.push(Token::KEYWORD(Keyword::For, self.next_start_line, self.next_start_col));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("continue") && Self::word_boundary(line, 8) {
            self.tokens.push(Token::KEYWORD(Keyword::Continue, self.next_start_line, self.next_start_col));
            self.next_start_col += 8;
            Ok(8)
        } else if line.starts_with_str("break") && Self::word_boundary(line, 5) {
            self.tokens.push(Token::KEYWORD(Keyword::Break, self.next_start_line, self.next_start_col));
            self.next_start_col += 5;
            Ok(5)
        } else if line.starts_with_str("return") && Self::word_boundary(line, 6) {
            self.tokens.push(Token::KEYWORD(Keyword::Return, self.next_start_line, self.next_start_col));
            self.next_start_col += 6;
            Ok(6)
        } else if line.starts_with_str("def") && Self::word_boundary(line, 3) {
            self.tokens.push(Token::KEYWORD(Keyword::Def, self.next_start_line, self.next_start_col));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("True") && Self::word_boundary(line, 4) {
            self.tokens.push(Token::BOOL(true, self.next_start_line, self.next_start_col));
            self.next_start_col += 4;
            Ok(4)
        } else if line.starts_with_str("False") && Self::word_boundary(line, 5) {
            self.tokens.push(Token::BOOL(false, self.next_start_line, self.next_start_col));
            self.next_start_col += 5;
            Ok(5)
        } else if line.starts_with_str("and") && Self::word_boundary(line, 3) {
            self.tokens.push(Token::OP(Op::And, self.next_start_line, self.next_start_col));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("or") && Self::word_boundary(line, 2) {
            self.tokens.push(Token::OP(Op::Or, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("in") && Self::word_boundary(line, 2) {
            self.tokens.push(Token::OP(Op::In, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("not in") && Self::word_boundary(line, 6) {
            self.tokens.push(Token::OP(Op::NotIn, self.next_start_line, self.next_start_col));
            self.next_start_col += 6;
            Ok(6)
        } else if line.starts_with_str("not") && Self::word_boundary(line, 3) {
            self.tokens.push(Token::OP(Op::Not, self.next_start_line, self.next_start_col));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("+=") {
            self.tokens.push(Token::ASOP(Asop::AddAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("-=") {
            self.tokens.push(Token::ASOP(Asop::SubAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("*=") {
            self.tokens.push(Token::ASOP(Asop::MultAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("/=") {
            self.tokens.push(Token::ASOP(Asop::DivAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("//=") {
            self.tokens.push(Token::ASOP(Asop::IntDivAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("%=") {
            self.tokens.push(Token::ASOP(Asop::ModAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("**=") {
            self.tokens.push(Token::ASOP(Asop::ExpAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("&=") {
            self.tokens.push(Token::ASOP(Asop::BWAndAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("|=") {
            self.tokens.push(Token::ASOP(Asop::BWOrAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("~=") {
            self.tokens.push(Token::ASOP(Asop::BWNotAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("^=") {
            self.tokens.push(Token::ASOP(Asop::XorAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("<<=") {
            self.tokens.push(Token::ASOP(Asop::ShLeftAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str(">>=") {
            self.tokens.push(Token::ASOP(Asop::ShRightAssign, self.next_start_line, self.next_start_col));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("+") {
            self.tokens.push(Token::OP(Op::Plus, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("-") {
            self.tokens.push(Token::OP(Op::Minus, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("**") {
            self.tokens.push(Token::OP(Op::Exp, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("*") {
            self.tokens.push(Token::OP(Op::Mult, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("//") {
            self.tokens.push(Token::OP(Op::IntDiv, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("/") {
            self.tokens.push(Token::OP(Op::Div, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("%") {
            self.tokens.push(Token::OP(Op::Mod, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("==") {
            self.tokens.push(Token::OP(Op::Eq, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("=") {
            self.tokens.push(Token::ASOP(Asop::Assign, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("!=") {
            self.tokens.push(Token::OP(Op::Neq, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("<<") {
            self.tokens.push(Token::OP(Op::ShLeft, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("<=") {
            self.tokens.push(Token::OP(Op::Lte, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("<") {
            self.tokens.push(Token::OP(Op::Lt, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str(">>") {
            self.tokens.push(Token::OP(Op::ShRight, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str(">=") {
            self.tokens.push(Token::OP(Op::Gte, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str(">") {
            self.tokens.push(Token::OP(Op::Gt, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("==") {
            self.tokens.push(Token::OP(Op::Eq, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("&") {
            self.tokens.push(Token::OP(Op::BWAnd, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("|") {
            self.tokens.push(Token::OP(Op::BWOr, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("^") {
            self.tokens.push(Token::OP(Op::Xor, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("~") {
            self.tokens.push(Token::OP(Op::BWNot, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("(") {
            self.tokens.push(Token::BRACKET('(', self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str(")") {
            self.tokens.push(Token::BRACKET(')', self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("[") {
            self.tokens.push(Token::BRACKET('[', self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("]") {
            self.tokens.push(Token::BRACKET(']', self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("{") {
            self.tokens.push(Token::BRACKET('{', self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("}") {
            self.tokens.push(Token::BRACKET('}', self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line[0].is_ascii_digit() { // number
            let mut idx = 1;
            let mut decimal_found = false;
            while !Self::number_boundary(line, idx) {
                if line[idx] == '.' {
                    if decimal_found {
                        return Err("malformed number (cannot have multiple decimal points)".to_string());
                    } else {
                        decimal_found = true;
                    }
                }
                idx += 1;
            }

            // Check for valid next character
            if idx < line.len() && (line[idx] != ' ' && !SYMBOLS.contains(&line[idx])) {
                return Err("malformed number (cannot contain non-numerical characters)".to_string());
            }

            self.tokens.push(Token::NUMBER(match line[..idx].iter().collect::<String>().parse::<f64>() {
                Ok(n) => n,
                Err(e) => {
                    return Err(format!(
                        "malformed number ({e})"
                    ));
                }
            }, self.next_start_line, self.next_start_col));
            self.next_start_col += idx;
            Ok(idx)
        } else if line[0] == '"' || line[0] == '\'' { // string
            let mut idx = 1;
            let max_idx = line.len() - 1;
            if max_idx > 1 {
                while line[idx] != line[0] { // Final quote needs to match the first one
                    idx += 1;
                    if idx >= max_idx {
                        return Err("malformed string (quote not closed)".to_string());
                    }
                }
            }

            self.tokens.push(Token::STRING(line[1..idx].iter().collect::<String>(), self.next_start_line, self.next_start_col));
            self.next_start_col += idx + 1;
            Ok(idx + 1)
        } else if line[0].is_ascii_alphabetic() { // name
            let mut idx = 1;
            for _ in &line[1..] {
                if Self::word_boundary(line, idx) {
                    break;
                }
                idx += 1;
            }

            self.tokens.push(Token::NAME(line[..idx].iter().collect::<String>(), self.next_start_line, self.next_start_col));
            self.next_start_col += idx;
            Ok(idx)
        } else { // misc
            self.tokens.push(Token::MISC(line[0], self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        }
        // == Tokenization logic ends here == //
    }

    fn word_boundary(line: &[char], idx: usize) -> bool {
        idx >= line.len() || (line[idx] != '_' && !line[idx].is_ascii_alphanumeric())
    }

    fn number_boundary(line: &[char], idx: usize) -> bool {
        idx >= line.len() || (line[idx] != '.' && !line[idx].is_digit(10))
    }
}
