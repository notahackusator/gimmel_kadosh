use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::rc::Rc;

use crate::lexer::prelude::{Token, TokenType};
use crate::parser::errors::ParseError;
use crate::parser::parseable::{Parseable, ParseResult};

#[derive(Clone, PartialEq)]
pub struct TokenIter<'a> {
    pub tokens: &'a [Token],
    pub index: usize,
    pub start: Vec<usize>
}

impl<'a> TokenIter<'a> {
    pub fn new(tokens: &'a [Token]) -> TokenIter<'a> {
        TokenIter { tokens, index: 0, start: vec![] }
    }

    pub fn has_next(&self) -> bool {
        self.index < self.tokens.len()
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.index >= self.tokens.len() {
            None
        } else {
            self.index += 1;
            Some(self.tokens[self.index - 1].clone())
        }
    }

    pub fn borrow_next(&mut self) -> Option<&'a Token> {
        if self.index >= self.tokens.len() {
            None
        } else {
            self.index += 1;
            Some(&self.tokens[self.index - 1])
        }
    }
}

macro_rules! parse_start {
    ($token_iter:ident $start_index:ident) => {
        let $start_index = $token_iter.index;
    };
}

macro_rules! result {
    ($token_iter:ident $start_index:ident $result:expr) => {
        {$token_iter.index = $start_index;
        return $result;}
    };
}

pub(crate) use {parse_start, result};

impl<'a> Debug for TokenIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad("TokenIter {\n")?;
        for token in &self.tokens[self.index..] {
            f.pad(format!("\t{token:?}\n").as_str())?;
        }
        f.pad("}")
    }
}