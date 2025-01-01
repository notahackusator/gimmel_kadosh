use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::condition;
use crate::lexer::prelude::Token;
use crate::parser::expression::Expression;
use crate::parser::token_iter::TokenIter;
use crate::parser::parseable::{Parseable, ParseResult};

use super::prelude::{Buf, ErrAction, Executor, huh, Param, TokenRegex, TokenRule, TokenSequence};

#[derive(Clone, Debug, PartialEq)]
pub struct Var {
    pub name: Token,
    pub value: Expression
}

impl Parseable for Var {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        let var_start_regex = TokenRegex::new(vec![
            TokenSequence::new(None, TokenRule::UniqueId("ויהי".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("משתנה".to_string()), vec![
                    TokenSequence::new(None, TokenRule::UniqueId("ושמו".to_string()), vec![
                        TokenSequence::new(Some("name".to_string()), TokenRule::Id, vec![
                            TokenSequence::new(None, TokenRule::UniqueId("וערכו".to_string()), vec![

                            ], Some("ציפה למילה 'וערכו'".to_string()))
                        ], Some("המשתנה ציפה לשם".to_string()))
                    ], Some("ציפה למילה 'ושמו'".to_string()))
                ], None)
            ], None),
            TokenSequence::new(None, TokenRule::UniqueId("יהי".to_string()), vec![
                TokenSequence::new(Some("name".to_string()), TokenRule::Id, vec![
                    TokenSequence::new(None, TokenRule::UniqueId("וערכו".to_string()), vec![

                    ], Some("ציפה למילה 'וערכו'".to_string()))
                ], None)
            ], None)
        ]);
        let map = huh!(var_start_regex.try_parse(token_iter));
        #[allow(suspicious_double_ref_op)]
        let name = map.get("name").unwrap().clone().clone();
        let value = huh!(Expression::try_parse(token_iter));
        ParseResult::Ok(Self {
            name,
            value,
        })
    }
}