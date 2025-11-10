use super::building_blocks::*;

const SYMBOLS: [char; 21] = [
    '+', '-', '*', '/', '%', '!', '>', '<', '&', '|', '^', '~', '=', '(', ')', '{', '}', '[', ']',
    ',', ':',
];

/// Convenience trait to allow the many `(&[char]).starts_with(&str)` invocations in
/// `Lexer::identify()`.
trait StartsWithStr {
    fn starts_with_str(&self, needle: &str) -> bool;
}

impl StartsWithStr for &[char] {
    fn starts_with_str(&self, needle: &str) -> bool {
        if needle.len() > self.len() {
            return false
        }
        self.iter().zip(needle.chars()).all(|(l, r)| l == &r)
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

    pub fn finalize(&mut self) -> Result<&Vec<Token>, String> {
        if self.finished {
            return Err("this lexer has finished its job".to_string());
        } else if let Token::NEWLINE(_, _) = self.tokens.last().unwrap_or(&Token::END) {
            // Don't push another newline if there already is one
            self.tokens.push(Token::END);
        } else {
            // Push an extra newline before the end because the grammar requires it
            self.tokens
                .push(Token::NEWLINE(self.next_start_line, self.next_start_col));
            self.tokens.push(Token::END);
        }

        self.finished = true;
        Ok(&self.tokens)
    }

    /// Used to advance a character iterator by lexeme. It identifies the lexeme, appends its lexed `Token` value to
    /// `self.tokens`, and returns how many characters the iterator was advanced by.
    ///
    /// Returns a `Err(String)` if something couldn't be lexed properly.
    pub fn identify(&mut self, line: &[char]) -> Result<usize, String> {
        if self.finished {
            return Err("this lexer has finished its job".to_string());
        }

        // Start all lines with an INDENT token, even if the amount is 0
        if self.next_start_col == 0 && !line.is_empty() && line[0] != ' ' && !line.starts_with_str("#") {
            self.tokens.push(Token::INDENT(0, self.next_start_line, 0));
        }

        // == Actual tokenization logic starts here == //
        if line.is_empty() {
            // newline
            self.tokens
                .push(Token::NEWLINE(self.next_start_line, self.next_start_col));
            self.next_start_col = 0;
            self.next_start_line += 1;
            Ok(1)
        } else if line[0] == ' ' {
            if self.next_start_col == 0 {
                // Count indentation spaces at the start of a line
                let mut num_spaces = 0;

                // Find how many spaces the line starts with
                for c in line {
                    if *c == ' ' {
                        num_spaces += 1;
                    } else if *c == '#' {
                        // We don't care about indentations if the line is only a comment
                        self.tokens
                            .push(Token::NEWLINE(self.next_start_line, self.next_start_col));
                        self.next_start_line += 1;
                        self.next_start_col = 0;
                        return Ok(line.len() + 1);
                    } else {
                        break;
                    }
                }

                // We don't care about indentations if the line contains nothing else
                if num_spaces == line.len() {
                    self.next_start_line += 1;
                    return Ok(line.len() + 1);
                }

                // Make sure the amount of spaces is valid
                if num_spaces % 4 != 0 {
                    return Err("unknown amount of indentations, number of spaces should be a multiple of 4".to_string());
                }

                // Finalize the identification
                self.tokens
                    .push(Token::INDENT(num_spaces / 4, self.next_start_line, 0));
                self.next_start_col += num_spaces;
                Ok(num_spaces)
            } else {
                // Ignore random spaces inside a line
                let mut num_spaces = 1;

                // Count the spaces
                for c in &line[1..] {
                    if *c == ' ' {
                        num_spaces += 1;
                    } else if *c == '#' {
                        // Ignore the rest of the line if the spaces are followed by a comment
                        self.tokens
                            .push(Token::NEWLINE(self.next_start_line, self.next_start_col));
                        self.next_start_line += 1;
                        self.next_start_col = 0;
                        return Ok(line.len() + 1);
                    } else {
                        break;
                    }
                }

                self.next_start_col += num_spaces;
                Ok(num_spaces)
            }
        } else if line.starts_with_str("#") {
            // Ignore the rest of the line and push a NEWLINE
            self.tokens
                .push(Token::NEWLINE(self.next_start_line, self.next_start_col));
            self.next_start_col = 0;
            self.next_start_line += 1;
            Ok(0)
        } else if line.starts_with_str("if") && Self::word_boundary(line, 2) {
            self.tokens.push(Token::KEYWORD(
                Keyword::If,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("while") && Self::word_boundary(line, 5) {
            self.tokens.push(Token::KEYWORD(
                Keyword::While,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 5;
            Ok(5)
        } else if line.starts_with_str("for") && Self::word_boundary(line, 3) {
            self.tokens.push(Token::KEYWORD(
                Keyword::For,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("continue") && Self::word_boundary(line, 8) {
            self.tokens.push(Token::KEYWORD(
                Keyword::Continue,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 8;
            Ok(8)
        } else if line.starts_with_str("break") && Self::word_boundary(line, 5) {
            self.tokens.push(Token::KEYWORD(
                Keyword::Break,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 5;
            Ok(5)
        } else if line.starts_with_str("return") && Self::word_boundary(line, 6) {
            self.tokens.push(Token::KEYWORD(
                Keyword::Return,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 6;
            Ok(6)
        } else if line.starts_with_str("def") && Self::word_boundary(line, 3) {
            self.tokens.push(Token::KEYWORD(
                Keyword::Def,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("True") && Self::word_boundary(line, 4) {
            self.tokens
                .push(Token::BOOL(true, self.next_start_line, self.next_start_col));
            self.next_start_col += 4;
            Ok(4)
        } else if line.starts_with_str("False") && Self::word_boundary(line, 5) {
            self.tokens.push(Token::BOOL(
                false,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 5;
            Ok(5)
        } else if line.starts_with_str("and") && Self::word_boundary(line, 3) {
            self.tokens.push(Token::OP(
                Op::And,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("or") && Self::word_boundary(line, 2) {
            self.tokens
                .push(Token::OP(Op::Or, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("in") && Self::word_boundary(line, 2) {
            self.tokens
                .push(Token::OP(Op::In, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("not in") && Self::word_boundary(line, 6) {
            self.tokens.push(Token::OP(
                Op::NotIn,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 6;
            Ok(6)
        } else if line.starts_with_str("not") && Self::word_boundary(line, 3) {
            self.tokens.push(Token::OP(
                Op::Not,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("+=") {
            self.tokens.push(Token::ASOP(
                Asop::AddAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("-=") {
            self.tokens.push(Token::ASOP(
                Asop::SubAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("*=") {
            self.tokens.push(Token::ASOP(
                Asop::MultAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("/=") {
            self.tokens.push(Token::ASOP(
                Asop::DivAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("//=") {
            self.tokens.push(Token::ASOP(
                Asop::IntDivAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("%=") {
            self.tokens.push(Token::ASOP(
                Asop::ModAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("**=") {
            self.tokens.push(Token::ASOP(
                Asop::ExpAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("&=") {
            self.tokens.push(Token::ASOP(
                Asop::BWAndAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("|=") {
            self.tokens.push(Token::ASOP(
                Asop::BWOrAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("~=") {
            self.tokens.push(Token::ASOP(
                Asop::BWNotAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("^=") {
            self.tokens.push(Token::ASOP(
                Asop::XorAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("<<=") {
            self.tokens.push(Token::ASOP(
                Asop::ShLeftAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str(">>=") {
            self.tokens.push(Token::ASOP(
                Asop::ShRightAssign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 3;
            Ok(3)
        } else if line.starts_with_str("+") {
            self.tokens.push(Token::OP(
                Op::Plus,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("-") {
            self.tokens.push(Token::OP(
                Op::Minus,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("**") {
            self.tokens.push(Token::OP(
                Op::Exp,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("*") {
            self.tokens.push(Token::OP(
                Op::Mult,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("//") {
            self.tokens.push(Token::OP(
                Op::IntDiv,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("/") {
            self.tokens.push(Token::OP(
                Op::Div,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("%") {
            self.tokens.push(Token::OP(
                Op::Mod,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("==") {
            self.tokens
                .push(Token::OP(Op::Eq, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("=") {
            self.tokens.push(Token::ASOP(
                Asop::Assign,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("!=") {
            self.tokens.push(Token::OP(
                Op::Neq,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("<<") {
            self.tokens.push(Token::OP(
                Op::ShLeft,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("<=") {
            self.tokens.push(Token::OP(
                Op::Lte,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("<") {
            self.tokens
                .push(Token::OP(Op::Lt, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str(">>") {
            self.tokens.push(Token::OP(
                Op::ShRight,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str(">=") {
            self.tokens.push(Token::OP(
                Op::Gte,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str(">") {
            self.tokens
                .push(Token::OP(Op::Gt, self.next_start_line, self.next_start_col));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("==") {
            self.tokens
                .push(Token::OP(Op::Eq, self.next_start_line, self.next_start_col));
            self.next_start_col += 2;
            Ok(2)
        } else if line.starts_with_str("&") {
            self.tokens.push(Token::OP(
                Op::BWAnd,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("|") {
            self.tokens.push(Token::OP(
                Op::BWOr,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("^") {
            self.tokens.push(Token::OP(
                Op::Xor,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("~") {
            self.tokens.push(Token::OP(
                Op::BWNot,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("(") {
            self.tokens.push(Token::BRACKET(
                '(',
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str(")") {
            self.tokens.push(Token::BRACKET(
                ')',
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("[") {
            self.tokens.push(Token::BRACKET(
                '[',
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("]") {
            self.tokens.push(Token::BRACKET(
                ']',
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("{") {
            self.tokens.push(Token::BRACKET(
                '{',
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line.starts_with_str("}") {
            self.tokens.push(Token::BRACKET(
                '}',
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        } else if line[0].is_ascii_digit() {
            // number

            let mut idx = 1;
            let mut decimal_found = false;
            while !Self::number_boundary(line, idx) {
                if line[idx] == '.' {
                    if decimal_found {
                        return Err(
                            "malformed number (cannot have multiple decimal points)".to_string()
                        );
                    } else {
                        decimal_found = true;
                    }
                }
                idx += 1;
            }

            // Check for valid next character
            if idx < line.len() && (line[idx] != ' ' && !SYMBOLS.contains(&line[idx])) {
                return Err(
                    "malformed number (cannot contain non-numerical characters)".to_string()
                );
            }

            self.tokens.push(Token::NUMBER(
                match line[..idx].iter().collect::<String>().parse::<f64>() {
                    Ok(n) => n,
                    Err(e) => {
                        return Err(format!("malformed number ({e})"));
                    }
                },
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += idx;
            Ok(idx)
        } else if line[0] == '"' || line[0] == '\'' {
            // string

            let mut result_str = String::new();
            let mut escaped = false;
            let mut idx = 1;
            let max_idx = line.len();
            if max_idx > 1 {
                while line[idx] != line[0] || escaped {
                    // Find first non-escaped matching quote
                    if escaped {
                        escaped = false;
                        result_str.push(line[idx]);
                    } else if line[idx] == '\\' {
                        escaped = true;
                    } else {
                        escaped = false;
                        result_str.push(line[idx]);
                    }

                    idx += 1;
                    if idx >= max_idx {
                        return Err("malformed string (quote not closed)".to_string());
                    }
                }
            } else {
                return Err("malformed string (quote not closed)".to_string());
            }

            self.tokens.push(Token::STRING(
                result_str,
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += idx + 1;
            Ok(idx + 1)
        } else if line[0].is_ascii_alphabetic() {
            // name

            let mut idx = 1;
            for _ in &line[1..] {
                if Self::word_boundary(line, idx) {
                    break;
                }
                idx += 1;
            }

            self.tokens.push(Token::NAME(
                line[..idx].iter().collect::<String>(),
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += idx;
            Ok(idx)
        } else {
            // misc

            self.tokens.push(Token::MISC(
                line[0],
                self.next_start_line,
                self.next_start_col,
            ));
            self.next_start_col += 1;
            Ok(1)
        }
        // == Tokenization logic ends here == //
    }

    fn word_boundary(line: &[char], idx: usize) -> bool {
        idx >= line.len() || (line[idx] != '_' && !line[idx].is_ascii_alphanumeric())
    }

    fn number_boundary(line: &[char], idx: usize) -> bool {
        idx >= line.len() || (line[idx] != '.' && !line[idx].is_ascii_digit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! char_slice {
        ($og:expr) => {
            &$og.chars().collect::<Vec<_>>()[..]
        };
    }

    #[test]
    fn test_starts_with_str() {
        let haystack = char_slice!("hello");
        assert!(haystack.starts_with_str("hel"));
        assert!(haystack.starts_with_str(""));
        assert!(haystack.starts_with_str("h"));
        assert!(haystack.starts_with_str("hello"));
        assert!(!haystack.starts_with_str("ello"));
        assert!(haystack.starts_with_str("hello world"));
    }

    #[test]
    fn test_lexer_full_usage() {
        let py_line = char_slice!("if x + y < 100:");
        let mut lexer = Lexer::new();

        // Check return values
        assert_eq!(lexer.identify(py_line), Ok(2)); // `indent(0) if`
        assert_eq!(lexer.identify(&py_line[2..]), Ok(1)); // ` `
        assert_eq!(lexer.identify(&py_line[3..]), Ok(1)); // `x`
        assert_eq!(lexer.identify(&py_line[4..]), Ok(1)); // ` `
        assert_eq!(lexer.identify(&py_line[5..]), Ok(1)); // `+`
        assert_eq!(lexer.identify(&py_line[6..]), Ok(1)); // ` `
        assert_eq!(lexer.identify(&py_line[7..]), Ok(1)); // `y`
        assert_eq!(lexer.identify(&py_line[8..]), Ok(1)); // ` `
        assert_eq!(lexer.identify(&py_line[9..]), Ok(1)); // `<`
        assert_eq!(lexer.identify(&py_line[10..]), Ok(1)); // ` `
        assert_eq!(lexer.identify(&py_line[11..]), Ok(3)); // `100`
        assert_eq!(lexer.identify(&py_line[14..]), Ok(1)); // `:`
        assert_eq!(lexer.identify(&py_line[15..]), Ok(1)); // newline

        // Check token stream
        let mut token_stream = lexer.finalize().unwrap().iter();
        assert_eq!(token_stream.next(), Some(&Token::INDENT(0, 0, 0)));
        assert_eq!(
            token_stream.next(),
            Some(&Token::KEYWORD(Keyword::If, 0, 0))
        );
        assert_eq!(
            token_stream.next(),
            Some(&Token::NAME("x".to_string(), 0, 3))
        );
        assert_eq!(token_stream.next(), Some(&Token::OP(Op::Plus, 0, 5)));
        assert_eq!(
            token_stream.next(),
            Some(&Token::NAME("y".to_string(), 0, 7))
        );
        assert_eq!(token_stream.next(), Some(&Token::OP(Op::Lt, 0, 9)));
        assert_eq!(token_stream.next(), Some(&Token::NUMBER(100.0, 0, 11)));
        assert_eq!(token_stream.next(), Some(&Token::MISC(':', 0, 14)));
        assert_eq!(token_stream.next(), Some(&Token::NEWLINE(0, 15)));
        assert_eq!(token_stream.next(), Some(&Token::END));
        assert_eq!(token_stream.next(), None);

        // Check lexer is done
        assert_eq!(
            lexer.identify(py_line).unwrap_err(),
            "this lexer has finished its job".to_string()
        );
        assert_eq!(
            lexer.finalize().unwrap_err(),
            "this lexer has finished its job".to_string()
        );
    }

    #[test]
    fn test_lexer_spaces() {
        // No spaces
        let mut lexer = Lexer::new();
        let py_line = char_slice!("x = 10");
        let mut col = 0;
        while col <= py_line.len() {
            col += lexer
                .identify(&py_line[col..])
                .expect("Should have identified successfully");
        }

        // Only spaces (invalid indentation)
        let mut lexer = Lexer::new();
        let py_line = char_slice!("   ");
        let mut col = 0;
        while col <= py_line.len() {
            col += lexer
                .identify(&py_line[col..])
                .expect("Should have identified successfully");
        }

        // Only spaces (valid indentation)
        let mut lexer = Lexer::new();
        let py_line = char_slice!("    ");
        let mut col = 0;
        while col <= py_line.len() {
            col += lexer
                .identify(&py_line[col..])
                .expect("Should have identified successfully");
        }

        // Spaces with comment (invalid indentation)
        let mut lexer = Lexer::new();
        let py_line = char_slice!("     # this is a comment");
        let mut col = 0;
        while col <= py_line.len() {
            col += lexer
                .identify(&py_line[col..])
                .expect("Should have identified successfully");
        }

        // Spaces with comment (valid indentation)
        let mut lexer = Lexer::new();
        let py_line = char_slice!("    # this is a comment");
        let mut col = 0;
        while col <= py_line.len() {
            col += lexer
                .identify(&py_line[col..])
                .expect("Should have identified successfully");
        }

        // Spaces inside the line
        let mut lexer = Lexer::new();
        let py_line = char_slice!("x         = 10");
        let mut col = 0;
        while col <= py_line.len() {
            col += lexer
                .identify(&py_line[col..])
                .expect("Should have identified successfully");
        }

        // Valid indentation
        let mut lexer = Lexer::new();
        let py_line = char_slice!("    x = 10");
        let mut col = 0;
        while col <= py_line.len() {
            col += lexer
                .identify(&py_line[col..])
                .expect("Should have identified successfully");
        }

        // Invalid indentation
        let mut lexer = Lexer::new();
        let py_line = char_slice!("   x = 10");
        assert_eq!(
            lexer.identify(py_line).unwrap_err(),
            "unknown amount of indentations, number of spaces should be a multiple of 4"
        );
    }

    #[test]
    fn test_lexer_numbers() {
        // Integer
        let mut lexer = Lexer::new();
        let py_line = char_slice!("156");
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(token_stream.next(), Some(&Token::NUMBER(156.0, 0, 0)));

        // Decimal number
        let mut lexer = Lexer::new();
        let py_line = char_slice!("156.89");
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(token_stream.next(), Some(&Token::NUMBER(156.89, 0, 0)));

        // Zero
        let mut lexer = Lexer::new();
        let py_line = char_slice!("0");
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(token_stream.next(), Some(&Token::NUMBER(0.0, 0, 0)));

        // Leading zeroes
        let mut lexer = Lexer::new();
        let py_line = char_slice!("0000017");
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(token_stream.next(), Some(&Token::NUMBER(17.0, 0, 0)));

        // Trailing zeroes
        let mut lexer = Lexer::new();
        let py_line = char_slice!("17.10000");
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(token_stream.next(), Some(&Token::NUMBER(17.1, 0, 0)));

        // More than one decimal point
        let mut lexer = Lexer::new();
        let py_line = char_slice!("156.1.0");
        lexer.identify(py_line).expect_err("should not compile");

        // Non-numerical characters
        let mut lexer = Lexer::new();
        let py_line = char_slice!("156ab");
        lexer.identify(py_line).expect_err("should not compile");
    }

    #[test]
    fn test_lexer_strings() {
        // To facilitate building strings
        let double_quote = "\"";
        let single_quote = "'";
        let escape = "\\";

        // Double-quoted
        let mut lexer = Lexer::new();
        let py_line = char_slice!(format!("{double_quote}hello world{double_quote}"));
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(
            token_stream.next(),
            Some(&Token::STRING("hello world".to_string(), 0, 0))
        );

        // Single-quoted
        let mut lexer = Lexer::new();
        let py_line = char_slice!(format!("{single_quote}hello world{single_quote}"));
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(
            token_stream.next(),
            Some(&Token::STRING("hello world".to_string(), 0, 0))
        );

        // Empty string
        let mut lexer = Lexer::new();
        let py_line = char_slice!(format!("{double_quote}{double_quote}"));
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(
            token_stream.next(),
            Some(&Token::STRING("".to_string(), 0, 0))
        );

        // Escaped double-quote
        let mut lexer = Lexer::new();
        let py_line = char_slice!(format!(
            "{double_quote}{escape}{double_quote}{double_quote}"
        )); // Looks like `\"`
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(
            token_stream.next(),
            Some(&Token::STRING("\"".to_string(), 0, 0))
        );

        // Escaped back-slash
        let mut lexer = Lexer::new();
        let py_line = char_slice!(format!("{double_quote}{escape}{escape}{double_quote}")); // Looks like `\\"`
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(
            token_stream.next(),
            Some(&Token::STRING("\\".to_string(), 0, 0))
        );

        // Unterminated double-quote
        let mut lexer = Lexer::new();
        let py_line = char_slice!(format!("{double_quote}"));
        lexer.identify(py_line).expect_err("should not compile");

        // Unterminated single-quote
        let mut lexer = Lexer::new();
        let py_line = char_slice!(format!("{single_quote}"));
        lexer.identify(py_line).expect_err("should not compile");

        // Mixed quotes
        let mut lexer = Lexer::new();
        let py_line = char_slice!(format!("{double_quote}{single_quote}"));
        lexer.identify(py_line).expect_err("should not compile");
    }

    #[test]
    fn test_lexer_names() {
        // Normal variable
        let mut lexer = Lexer::new();
        let py_line = char_slice!("var");
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(
            token_stream.next(),
            Some(&Token::NAME("var".to_string(), 0, 0))
        );

        // With underscores
        let mut lexer = Lexer::new();
        let py_line = char_slice!("my_var_name");
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(
            token_stream.next(),
            Some(&Token::NAME("my_var_name".to_string(), 0, 0))
        );

        // With digits
        let mut lexer = Lexer::new();
        let py_line = char_slice!("var123");
        lexer.identify(py_line).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(
            token_stream.next(),
            Some(&Token::NAME("var123".to_string(), 0, 0))
        );

        // With period
        let mut lexer = Lexer::new();
        let py_line = char_slice!("var.func()");
        lexer.identify(py_line).unwrap();
        lexer.identify(&py_line[3..]).unwrap();
        let mut token_stream = lexer.finalize().unwrap().iter();
        token_stream.next(); // First token is an empty INDENT
        assert_eq!(
            token_stream.next(),
            Some(&Token::NAME("var".to_string(), 0, 0))
        );
        assert_eq!(token_stream.next(), Some(&Token::MISC('.', 0, 3)));
    }

    #[test]
    #[ignore = "I'm too lazy to test every single token, maybe I'll do it later"]
    fn test_lexer_exhaustive() {
        todo!();
    }
}
