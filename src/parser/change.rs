use crate::lexer::prelude::*;
use crate::parser::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub name: Token,
    pub value: Expression
}

impl Parseable for Change {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let change_start_regex = TokenRegex::new(vec![TokenSequence::new(
            None, TokenRule::UniqueId("שנה".to_string()), vec![TokenSequence::new(
                None, TokenRule::UniqueId("את".to_string()), vec![TokenSequence::new(
                    Some("name".to_string()), TokenRule::Id, vec![TokenSequence::new(
                        None, TokenRule::UniqueId("להיות".to_string()), vec![], Some("ציפה למילה 'להיות'".to_string())
                    )], Some("ציפה לשם לאחר 'את'".to_string())
                )], Some("ציפה למילה 'את'".to_string())
            )], None
        )]);
        let map = chain_err!(token_iter start change_start_regex.try_parse(token_iter),
            err: ParseError::new("בתוך שינוי"));
        #[allow(suspicious_double_ref_op)]
        let name = map.get("name").unwrap().clone().clone();
        let value = chain_err!(token_iter start Expression::try_parse(token_iter),
            err: ParseError::new("בתוך שינוי"));
        ParseResult::Ok(Self {
            name,
            value,
        })
    }
}