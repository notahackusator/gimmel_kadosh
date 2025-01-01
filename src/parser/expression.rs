use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::condition;
use crate::lexer::prelude::{Token, TokenType};
use crate::parser::errors::ParseError;
use crate::parser::token_iter::TokenIter;
use crate::parser::parseable::{huh, Parseable, ParseResult};

use super::prelude::{Buf, ErrAction, Executor, Param, TokenEnclosing, TokenRegex, TokenRule, TokenSequence};

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub base_values: Box<[BaseValue]>,
    pub operators: Box<[Operator]>
}

impl Parseable for Expression {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        let first = huh!(BaseValue::try_parse(token_iter));
        let mut base_values = vec![first];
        let mut operators = vec![];

        let end_regex = TokenRegex::new(vec![
            TokenSequence::new(None, TokenRule::Divider(":".to_string()), vec![], None),
            TokenSequence::new(None, TokenRule::Divider(",".to_string()), vec![], None)
        ]);
        while token_iter.has_next() {
            if end_regex.try_parse(token_iter).is_ok() {
                token_iter.index -= 1;
                break;
            }

            let operator = huh!(Operator::try_parse(token_iter));
            let base_value = huh!(BaseValue::try_parse(token_iter));

            operators.push(operator);
            base_values.push(base_value);
        }

        ParseResult::Ok(Self {
            base_values: base_values.into(),
            operators: operators.into()
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BaseValue {
    Value {
        token: Token
    },
    FunctionCall {
        name: Token,
        parameters: Box<[Expression]>
    }
}

impl Parseable for BaseValue {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        let name_regex = TokenRegex::new(vec![
            TokenSequence::new(Some("name".to_string()), TokenRule::Id, vec![], None),
            TokenSequence::new(Some("value".to_string()), TokenRule::Value, vec![], None)
        ]);
        let name = match name_regex.try_parse(token_iter) {
            ParseResult::Ok(map) => {
                #[allow(suspicious_double_ref_op)] // <--
                if let Some(name) = map.get("name") {
                    name.clone()
                } else {
                    return ParseResult::Ok(Self::Value {
                        token: map.get("value").unwrap().clone().clone() // <--
                    });
                }
            }
            ParseResult::Err(err) => return ParseResult::Err(err),
            ParseResult::Skip => return ParseResult::Skip
        };
        let comma = TokenRule::Divider(",".to_string());

        let parentheses = TokenEnclosing::parentheses();
        let mut param_iter = match parentheses.try_parse(token_iter) {
            ParseResult::Ok(params) => params,
            ParseResult::Err(errors) => {
                let err = &errors[0];
                return if err.get_reason() == &parentheses.open_fail {
                    ParseResult::Ok(Self::Value {
                        token: name.clone(),
                    })
                } else {
                    ParseResult::Err(errors)
                }
            }
            ParseResult::Skip => unreachable!()
        };

        let mut params = vec![];
        while param_iter.has_next() {
            match Expression::try_parse(&mut param_iter) {
                ParseResult::Ok(expr) => params.push(expr),
                ParseResult::Err(err) => return ParseResult::Err(err),
                ParseResult::Skip => unreachable!()
            }

            if !param_iter.has_next() {
                break;
            }

            let next = param_iter.next().unwrap();
            if !comma.matches(&next.token_type) {
                return ParseResult::Err(vec![ParseError::indexed(
                    format!("Expected comma, found {}", next.token_type),
                    param_iter.index - 1
                )]);
            }
        }

        ParseResult::Ok(Self::FunctionCall {
            name: name.clone(),
            parameters: params.into_boxed_slice()
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod
}

impl Parseable for Operator {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        let token = match token_iter.next() {
            None => return ParseResult::Err(vec![ParseError::indexed(
                "Expected operator, but no tokens were found", token_iter.index)]),
            Some(token) => token
        };

        if let TokenType::Identifier(div) = token.token_type {
            match div.as_str() {
                "ועוד" => ParseResult::Ok(Self::Add),
                "פחות" => ParseResult::Ok(Self::Sub),
                "כפול" => ParseResult::Ok(Self::Mul),
                "חלקי" => ParseResult::Ok(Self::Div),
                "שארית" => ParseResult::Ok(Self::Mod),
                _ => ParseResult::Skip
            }
        } else {
            ParseResult::Skip
        }
    }
}