mod building_blocks;
mod lexer;
mod ptag;
mod tpg;

use std::{fmt::Display, fs, sync::OnceLock};

use colored::Colorize;

static FILENAME: OnceLock<String> = OnceLock::new();
static LINES: OnceLock<Vec<String>> = OnceLock::new();

#[derive(Debug)]
enum ParseErrorType {
    Marked {
        filename: String,
        line: usize,
        col: usize,
        line_string: String,
    },
    General
}

#[derive(Debug)]
pub struct ParseError {
    err_type: ParseErrorType,
    pub msg: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.err_type {
            ParseErrorType::Marked{filename, line, col, line_string} => {
                let location = format!("{filename}:{}:{}", line+1, col+1);
                let cursor = str::repeat(" ", col+1) + "^";
                f.write_str(&format!(
                    "({location}) {} {}\n  {} {line_string}\n   {}", 
                    "error:".red().bold(), 
                    self.msg.bold(),
                    "|".blue(),
                    cursor.red().bold()
                ))
            },
            ParseErrorType::General => {
                f.write_str(&format!(
                    "{}: {}",
                    "error".red().bold(),
                    self.msg.bold()
                ))
            }
        }
    }
}

impl ParseError {
    pub fn general(msg: &str) -> Self {
        Self {
            err_type: ParseErrorType::General,
            msg: msg.to_string()
        }
    }

    pub fn marked(msg: &str, line: usize, col: usize) -> Self {
        let filename = FILENAME.get_or_init(|| "unset".to_string());
        let line_string = match LINES.get() {
            Some(s) => &s[line],
            None => {
                return Self::general("Fatal error: lines were never set")
            }
        };

        Self {
            err_type: ParseErrorType::Marked {
                filename: filename.clone(),
                line,
                col, 
                line_string: line_string.clone()
            },
            msg: msg.to_string()
        }
    }
}

#[derive(Debug, Default)]
pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_from_file(&mut self, filename: &str) -> Result<(), ParseError> {
        FILENAME.set(filename.to_string()).unwrap();
        let script = fs::read_to_string(filename)
            .map_err(|e| ParseError::general(&e.to_string()))?;

        self.parse_from_str(&script)
    }

    pub fn parse_from_str(&mut self, script: &str) -> Result<(), ParseError> {
        let mut lex = lexer::Lexer::new();

        LINES.set(script.lines()
            .map(|l| l.to_string())
            .collect()).unwrap();

        for (line, line_str) in LINES.get().unwrap().iter().enumerate() {
            let line_chars = line_str.chars()
                .collect::<Vec<char>>();
            // Not `line_str.len() - 1` because we want to count the excluded newline 
            let max_col = line_str.len();
            let mut col = 0;

            // Keep identifying lexemes until the line is finished being scanned.
            // `col` goes up to AND INCLUDING `max_col` to account for the newline, which
            // is not included in the char slice.
            while col <= max_col {
                col += lex.identify(&line_chars[col..])
                    .map_err(|e| {
                        ParseError::marked(
                            &e, 
                            line, 
                            col
                        )
                    })?;
            }
        }

        let token_stream = dbg!(lex.finalize().map_err(|e| {
            ParseError::general(&e)
        })?);

        tpg::parse_tokens(token_stream).map(|tree| dbg!(tree))?;

        Ok(())
    }
}
