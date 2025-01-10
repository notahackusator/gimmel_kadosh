use std::collections::HashMap;
use crate::lexer::prelude::*;
use crate::parser::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    pub things: Vec<Path>,
    pub from: Path
}

impl Parseable for Import {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let start_regex = TokenRegex::new(vec![TokenSequence::new(
            None, TokenRule::UniqueId("ייבא".to_string()), vec![
                TokenSequence::new(
                    Some("את".to_string()), TokenRule::UniqueId("את".to_string()), vec![

                    ], None
                ),
                TokenSequence::new(
                    Some("דבר".to_string()), TokenRule::Id, vec![

                    ], Some("ציפה למשהו להביא".to_string())
                )
            ], None
        )]);
        let everything_regex = TokenRegex::new(vec![TokenSequence::new(
            None, TokenRule::UniqueId("הכל".to_string()), vec![

            ], None
        )]);

        let map = chain_err!(token_iter start start_regex.try_parse(token_iter),
            err: ParseError::new("בתוך יבוא"));

        #[allow(suspicious_double_ref_op)]
        let mut things = if map.contains_key("את") {
            if everything_regex.try_parse(token_iter).is_ok() {
                vec![]
            } else {
                token_iter.index -= 1;
                vec![chain_err!(token_iter start Path::try_parse(token_iter),
                    err: ParseError::new("בתוך ייבוא"))]
            }
        } else if map.contains_key("דבר") {
            token_iter.index -= 1;
            vec![chain_err!(token_iter start Path::try_parse(token_iter),
                    err: ParseError::new("בתוך ייבוא"))]
        } else {
            unreachable!();
        };

        if !things.is_empty() {
            let comma = TokenRule::Divider(",".to_string());

            while token_iter.has_next() {
                let next = token_iter.next().unwrap();
                if !comma.matches(&next.token_type) {
                    token_iter.index -= 1;
                    break;
                }

                let path = chain_err!(token_iter start Path::try_parse(token_iter),
                err: ParseError::new("בתוך ייבוא"));

                things.push(path);
            }
        }

        let from_regex = TokenRegex::new(vec![TokenSequence::new(
            None, TokenRule::UniqueId("מתוך".to_string()), vec![

            ], Some("ציפה למילה 'מתוך'".to_string())
        )]);
        chain_err!(token_iter start from_regex.try_parse(token_iter),
            err: ParseError::new("בתוך יבוא"));

        let from = chain_err!(token_iter start Path::try_parse(token_iter),
            err: ParseError::new("בתוך יבוא"));

        ParseResult::Ok(Self {
            things,
            from,
        })
    }
}