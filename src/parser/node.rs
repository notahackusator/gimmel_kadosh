use crate::{lexer::prelude::Token, parser::var::Var};
use crate::parser::prelude::ParseError;
use super::{parseable::{Parseable, ParseResult}, prelude::TokenIter};

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Var(Var)
}

pub fn parse(code: &[Token]) -> ParseResult<Vec<Node>> {
    let mut nodes: Vec<Node> = vec![];
    let mut token_iter: TokenIter = TokenIter::new(code);
    while token_iter.has_next() {
        match Var::try_parse(&mut token_iter) {
            ParseResult::Ok(x) => {
                nodes.push(Node::Var(x));
                continue;
            },
            ParseResult::Err(vec) => return ParseResult::Err(vec),
            ParseResult::Skip => {},
        }
        return ParseResult::Err(vec![ParseError::indexed(
            "אסימון לא ידוע",
            token_iter.index
        )]);
    }
    ParseResult::Ok(nodes)
}