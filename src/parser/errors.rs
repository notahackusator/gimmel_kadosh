use std::fmt::Display;
use std::str::Chars;
use crate::lexer::prelude::{Token, TokenType};

#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    Indexed {
        reason: String,
        index: usize
    },
    Reason {
        reason: String
    }
}

impl ParseError {
    pub fn new<D>(reason: D) -> Self where D: Display {
        Self::Reason { reason: reason.to_string() }
    }

    pub fn indexed<D>(reason: D, index: usize) -> Self where D: Display {
        Self::Indexed { reason: reason.to_string(), index }
    }
}

pub trait FormatInCode {
    fn format(&self, tokens: &[Token], code: &str) -> String;
}

impl FormatInCode for Vec<ParseError> {
    fn format(&self, tokens: &[Token], code: &str) -> String {
        let mut lines: Vec<(&str, usize)> = Vec::new();

        let mut start: usize = 0;
        let mut i: usize = 0;
        let mut chars: Chars = code.chars();
        while let Some(ch) = chars.next() {
            if ch == '\n' {
                lines.push((&code[start..i], start));
                start = i;
            }
            i += 1;
        }
        lines.push((&code[start..i], start));

        let mut msg: String = String::new();
        for error in self {
            match error {
                ParseError::Indexed { reason, index } => {
                    msg.push_str(&reason);

                    let token: &Token =
                        if let Some(token) = tokens.get(*index) {
                            token
                        } else if let Some(last) = tokens.last() {
                            last
                        } else {
                            continue;
                        };

                    let line_start: usize = match token.line {
                        1 => 0,
                        line => lines[line - 1].1 + 1
                    };
                    let line_end: usize = lines.get(token.line).map(|(_, i)| *i).unwrap_or(code.len());
                    let line: &str = &code[line_start..line_end];
                    msg.push_str(&format!("\n line {}: {}\n", token.line, line));

                    let length: usize =
                        match &token.token_type {
                            TokenType::Identifier(id) => id.len(),
                            TokenType::Integer(int) => int.len(),
                            TokenType::Decimal(int, dec) => int.len() + dec.len() + 1,
                            TokenType::String(str) => str.len(),
                            TokenType::Char(ch) => ch.len(),
                            TokenType::Divider(div) => div.len(),
                            TokenType::Operator(op) => op.len()
                        };

                    msg.push_str(&" ".repeat(token.col + 7 + token.line.to_string().len()));
                    msg.push_str(&"^".repeat(length));
                }
                ParseError::Reason { reason } => msg.push_str(&reason),
            }
            msg.push('\n');
        }
        msg
    }
}