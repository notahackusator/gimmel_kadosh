use std::collections::HashMap;
use crate::lexer::prelude::*;
use crate::parser::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Break {
    pub loop_name: Option<Token>
}

impl Parseable for Break {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let regex = TokenRegex::new(vec![TokenSequence::new(
            None, TokenRule::UniqueId("עצור".to_string()), vec![], None
        )]);
        if regex.try_parse(token_iter).is_err() {
            result!(token_iter start ParseResult::Skip);
        }
        let name_regex = TokenRegex::new(vec![TokenSequence::new(
            Some("name".to_string()), TokenRule::Id, vec![], None
        )]);
        #[allow(suspicious_double_ref_op)]
        match name_regex.try_parse(token_iter) {
            ParseResult::Ok(map) => ParseResult::Ok(Self {
                loop_name: map.get("name").map(|token| token.clone().clone())
            }),
            ParseResult::Skip => ParseResult::Ok(Self {
                loop_name: None
            }),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Continue {
    pub loop_name: Option<Token>
}

impl Parseable for Continue {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let regex = TokenRegex::new(vec![TokenSequence::new(
            None, TokenRule::UniqueId("דלג".to_string()), vec![], None
        )]);
        if regex.try_parse(token_iter).is_err() {
            result!(token_iter start ParseResult::Skip);
        }
        let name_regex = TokenRegex::new(vec![TokenSequence::new(
            Some("name".to_string()), TokenRule::Id, vec![], None
        )]);
        #[allow(suspicious_double_ref_op)]
        match name_regex.try_parse(token_iter) {
            ParseResult::Ok(map) => ParseResult::Ok(Self {
                loop_name: map.get("name").map(|token| token.clone().clone())
            }),
            ParseResult::Skip => ParseResult::Ok(Self {
                loop_name: None
            }),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct While {
    pub condition: Expression,
    pub code: Vec<Node>
}

impl Parseable for While {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let regex = TokenRegex::new(vec![TokenSequence::new(
            None, TokenRule::UniqueId("כל".to_string()), vec![TokenSequence::new(
                None, TokenRule::UniqueId("עוד".to_string()), vec![

                ], Some("ציפה למילה 'כל'".to_string())
            )], None
        )]);

        chain_err!(token_iter start regex.try_parse(token_iter),
            err: ParseError::new("בתוך לולאה"));

        let condition = chain_err!(token_iter start Expression::try_parse(token_iter),
            err: ParseError::new("בתוך לולאה"));

        let mut code = vec![];

        let mut code_iter = chain_err!(token_iter start TokenEnclosing::parentheses().try_parse(token_iter),
            err: ParseError::new("בתוך לולאה"));
        while code_iter.has_next() {
            let node = Node::try_parse(&mut code_iter);

            code.push(chain_err!(token_iter start node,
                err: ParseError::new("בתוך לולאה")));
        }

        ParseResult::Ok(Self {
            condition,
            code
        })
    }
}