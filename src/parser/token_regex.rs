use std::collections::HashMap;
use crate::lexer::prelude::{Token, TokenType};
use crate::parser::errors::ParseError;
use crate::parser::prelude::{ParseResult, TokenIter};

pub enum TokenRule {
    Id,
    UniqueId(String),
    Value,
    Divider(String)
}

impl TokenRule {
    pub fn matches(&self, token_type: &TokenType) -> bool {
        match (self, token_type) {
            (TokenRule::Id, TokenType::Identifier(_)) => true,
            (TokenRule::UniqueId(id1), TokenType::Identifier(id2)) => id1 == id2,
            (TokenRule::Value, TokenType::Identifier(_) | TokenType::String(_) |
                    TokenType::Integer(_) | TokenType::Decimal(_, _) | TokenType::Char(_)) => true,
            (TokenRule::Divider(div1), TokenType::Divider(div2)) => div1 == div2,
            _ => false
        }
    }
}

pub struct TokenSequence {
    select: Option<String>,
    rule: TokenRule,
    branches: Vec<TokenSequence>,
    on_fail: Option<String>
}

impl TokenSequence {
    pub fn new(select: Option<String>,
               rule: TokenRule,
               branches: Vec<TokenSequence>,
               on_fail: Option<String>) -> Self {
        Self {
            select,
            rule,
            branches,
            on_fail
        }
    }

    fn try_parse_sequence<'a>(&self, token_iter: &mut TokenIter<'a>, map: &mut HashMap<String, &'a Token>) -> ParseResult<()> {
        let token = match token_iter.borrow_next() {
            None => return match &self.on_fail {
                None => ParseResult::Skip,
                Some(expected) => ParseResult::Err(vec![ParseError::indexed(
                    format!("{expected}, אך לא נמצאו אסימונים"),
                    token_iter.index
                )])
            },
            Some(token) => token
        };

        if !self.rule.matches(&token.token_type) {
            return match &self.on_fail {
                None => ParseResult::Skip,
                Some(expected) => ParseResult::Err(vec![ParseError::indexed(
                    format!("{expected}, found {}", &token.token_type),
                    token_iter.index - 1
                )])
            };
        }

        if let Some(select) = &self.select {
            map.insert(select.to_string(), token);
        }

        if self.branches.is_empty() {
            return ParseResult::Ok(());
        }

        for branch in &self.branches {
            match branch.try_parse_sequence(token_iter, map) {
                ParseResult::Ok(_) => return ParseResult::Ok(()),
                ParseResult::Err(err) => return ParseResult::Err(err),
                _ => {}
            }
        }

        ParseResult::Skip
    }
}

pub struct TokenRegex {
    branches: Vec<TokenSequence>
}

impl TokenRegex {
    pub fn new(branches: Vec<TokenSequence>) -> Self {
        Self { branches }
    }

    pub fn try_parse<'a>(&self, token_iter: &mut TokenIter<'a>) -> ParseResult<HashMap<String, &'a Token>> {
        let mut map = HashMap::new();
        match self.try_parse_internal(token_iter, &mut map) {
            ParseResult::Ok(_) => {
                ParseResult::Ok(map)
            }
            err => {
                err.map(|_| map)
            }
        }
    }

    fn try_parse_internal<'a>(&self, token_iter: &mut TokenIter<'a>, map: &mut HashMap<String, &'a Token>) -> ParseResult<()> {
        for branch in &self.branches {
            token_iter.mark_start();
            match branch.try_parse_sequence(token_iter, map) {
                ParseResult::Ok(_) => return ParseResult::Ok(()),
                ParseResult::Err(err) => {
                    token_iter.revert();
                    return ParseResult::Err(err)
                },
                _ => token_iter.revert()
            }
        }
        ParseResult::Skip
    }
}

pub struct TokenEnclosing {
    pub open: TokenRule,
    pub close: TokenRule,
    pub open_fail: String,
    pub close_fail: String
}

impl TokenEnclosing {
    pub fn parentheses() -> Self {
        Self {
            open: TokenRule::Divider("(".to_string()),
            close: TokenRule::Divider(")".to_string()),
            open_fail: "Expected '('".to_string(),
            close_fail: "Expected ')'".to_string()
        }
    }

    pub fn try_parse<'a>(&self, token_iter: &mut TokenIter<'a>) -> ParseResult<TokenIter<'a>> {
        token_iter.mark_start();
        let first = match token_iter.borrow_next() {
            None => return ParseResult::Err(vec![ParseError::indexed(&self.open_fail, token_iter.index)]),
            Some(token) => token
        };
        if !self.open.matches(&first.token_type) {
            token_iter.revert();
            return ParseResult::Err(vec![ParseError::indexed(&self.open_fail, token_iter.index)]); // original == current - 1
        }

        let mut start = token_iter.index;
        let mut num_open = 1u32;
        while num_open > 0 {
            let token = match token_iter.borrow_next() {
                None => {
                    let idx = token_iter.index;
                    token_iter.revert();
                    return ParseResult::Err(vec![ParseError::indexed(&self.close_fail, idx - 1)]);
                },
                Some(token) => token
            };

            if self.open.matches(&token.token_type) {
                num_open += 1;
            }
            if self.close.matches(&token.token_type) {
                num_open -= 1;
            }
        }

        ParseResult::Ok(TokenIter {
            tokens: &token_iter.tokens[..token_iter.index - 1],
            index: start,
            start: vec![],
        })
    }
}