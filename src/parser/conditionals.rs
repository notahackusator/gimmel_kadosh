use crate::lexer::prelude::Token;
use crate::parser::prelude::*;
use crate::parser::token_iter;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    pub(crate) condition: Expression,
    pub(crate) code: Vec<Node>
}

impl Parseable for If {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let token = match token_iter.next() {
            None => return ParseResult::Skip,
            Some(token) => token
        };
        let if_rule = TokenRule::UniqueId("אם".to_string());
        if !if_rule.matches(&token.token_type) {
            token_iter.index = start;
            return ParseResult::Skip;
        }

        let condition = chain_err!(token_iter start Expression::try_parse(token_iter),
            err: ParseError::new("בתוך תנאי"));
        let mut code = vec![];

        let mut code_iter = chain_err!(token_iter start TokenEnclosing::parentheses().try_parse(token_iter),
            err: ParseError::new("בתוך קוד מותנה"));
        while code_iter.has_next() {
            let node = Node::try_parse(&mut code_iter);

            code.push(chain_err!(token_iter start node,
                err: ParseError::new("בתוך קוד מותנה")));
        }

        ParseResult::Ok(Self {
            condition,
            code,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElseIf {
    pub level: u8,
    pub condition: Expression,
    pub code: Vec<Node>
}

impl Parseable for ElseIf {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let token = match token_iter.next() {
            None => return ParseResult::Skip,
            Some(token) => token
        };

        let if_rule = TokenRule::UniqueId("ואם".to_string());
        if !if_rule.matches(&token.token_type) {
            token_iter.index = start;
            return ParseResult::Skip;
        }

        let level_regexes = vec![
            TokenRegex::new(vec![TokenSequence::new(None, TokenRule::UniqueId("לא".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("אך".to_string()), vec![], None)
            ], None)]),
            TokenRegex::new(vec![TokenSequence::new(None, TokenRule::UniqueId("גם".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("לא".to_string()), vec![
                    TokenSequence::new(None, TokenRule::UniqueId("אך".to_string()), vec![], None)
                ], None),
            ], None)]),
            TokenRegex::new(vec![TokenSequence::new(None, TokenRule::UniqueId("עדיין".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("לא".to_string()), vec![
                    TokenSequence::new(None, TokenRule::UniqueId("אך".to_string()), vec![], None)
                ], None),
            ], None)]),
            TokenRegex::new(vec![TokenSequence::new(None, TokenRule::UniqueId("אפילו".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("עדיין".to_string()), vec![
                    TokenSequence::new(None, TokenRule::UniqueId("לא".to_string()), vec![
                        TokenSequence::new(None, TokenRule::UniqueId("אך".to_string()), vec![], None)
                    ], None),
                ], None)
            ], None)])
        ];

        let mut level = 100u8;
        for (i, regex) in level_regexes.iter().enumerate() {
            if regex.try_parse(token_iter).is_ok() {
                level = i as u8;
                break;
            }
        }

        if level == 100 {
            token_iter.index = start;
            return ParseResult::Skip;
        }

        let condition = chain_err!(token_iter start Expression::try_parse(token_iter),
            err: ParseError::indexed("בתוך תנאי", token_iter.index));
        let mut code = vec![];

        let mut code_iter = chain_err!(token_iter start TokenEnclosing::parentheses().try_parse(token_iter),
            err: ParseError::new("בתוך קוד מותנה"));
        while code_iter.has_next() {
            let node = Node::try_parse(&mut code_iter);

            code.push(chain_err!(token_iter start node,
                err: ParseError::new("בתוך קוד מותנה")));
        }

        ParseResult::Ok(Self {
            level,
            condition,
            code,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Else {
    pub level: u8,
    pub code: Vec<Node>
}

impl Parseable for Else {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let token = match token_iter.next() {
            None => return ParseResult::Skip,
            Some(token) => token
        };

        let if_rule = TokenRule::UniqueId("ואם".to_string());
        if !if_rule.matches(&token.token_type) {
            token_iter.index = start;
            return ParseResult::Skip;
        }

        let level_regexes = vec![
            TokenRegex::new(vec![TokenSequence::new(None, TokenRule::UniqueId("לא".to_string()), vec![], None)]),
            TokenRegex::new(vec![TokenSequence::new(None, TokenRule::UniqueId("גם".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("לא".to_string()), vec![], None),
            ], None)]),
            TokenRegex::new(vec![TokenSequence::new(None, TokenRule::UniqueId("עדיין".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("לא".to_string()), vec![], None),
            ], None)]),
            TokenRegex::new(vec![TokenSequence::new(None, TokenRule::UniqueId("אפילו".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("עדיין".to_string()), vec![
                    TokenSequence::new(None, TokenRule::UniqueId("לא".to_string()), vec![], None),
                ], None)
            ], None)])
        ];

        let mut level = 100u8;
        for (i, regex) in level_regexes.iter().enumerate() {
            if regex.try_parse(token_iter).is_ok() {
                level = i as u8;
                break;
            }
        }

        if level == 100 {
            token_iter.index = start;
            return ParseResult::Skip;
        }

        let mut code = vec![];

        let mut code_iter = chain_err!(token_iter start TokenEnclosing::parentheses().try_parse(token_iter),
            err: ParseError::new("בתוך קוד מותנה"));
        while code_iter.has_next() {
            let node = Node::try_parse(&mut code_iter);

            code.push(chain_err!(token_iter start node,
                err: ParseError::new("בתוך קוד מותנה")));
        }

        ParseResult::Ok(Self {
            level,
            code,
        })
    }
}