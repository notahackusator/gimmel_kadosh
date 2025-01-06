use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::lexer::prelude::*;
use crate::parser::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Var {
    pub name: Token,
    pub value: Expression
}

impl Parseable for Var {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
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
        let map = chain_err!(token_iter start var_start_regex.try_parse(token_iter),
            err: ParseError::new("בתוך משתנה"));
        #[allow(suspicious_double_ref_op)]
        let name = map.get("name").unwrap().clone().clone();
        let value = chain_err!(token_iter start Expression::try_parse(token_iter),
            err: ParseError::new("בתוך משתנה"));
        ParseResult::Ok(Self {
            name,
            value,
        })
    }
}