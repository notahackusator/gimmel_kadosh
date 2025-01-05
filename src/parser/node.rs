use crate::lexer::prelude::*;
use crate::parser::prelude::*;

pub struct Eol;

impl Parseable for Eol {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        let end_regex = TokenRegex::new(vec![
            TokenSequence::new(None, TokenRule::Divider(":".to_string()), vec![], None),
            TokenSequence::new(None, TokenRule::Divider(",".to_string()), vec![], None),
            TokenSequence::new(None, TokenRule::Divider(";".to_string()), vec![], None),
            TokenSequence::new(None, TokenRule::Divider("(".to_string()), vec![], None)
        ]);

        end_regex.try_parse(token_iter).map(|_| Self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Var(Var),
    If(If),
    ElseIf(ElseIf),
    Else(Else),
    Expression(Expression),
    Eol,
}

impl Parseable for Node {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        match Var::try_parse(token_iter) {
            ParseResult::Ok(x) => return ParseResult::Ok(Self::Var(x)),
            ParseResult::Err(vec) => return ParseResult::Err(vec),
            ParseResult::Skip => {},
        }
        match If::try_parse(token_iter) {
            ParseResult::Ok(x) => return ParseResult::Ok(Self::If(x)),
            ParseResult::Err(vec) => return ParseResult::Err(vec),
            ParseResult::Skip => {},
        }
        match ElseIf::try_parse(token_iter) {
            ParseResult::Ok(x) => return ParseResult::Ok(Self::ElseIf(x)),
            ParseResult::Err(vec) => return ParseResult::Err(vec),
            ParseResult::Skip => {},
        }
        match Else::try_parse(token_iter) {
            ParseResult::Ok(x) => return ParseResult::Ok(Self::Else(x)),
            ParseResult::Err(vec) => return ParseResult::Err(vec),
            ParseResult::Skip => {},
        }
        match Expression::try_parse(token_iter) {
            ParseResult::Ok(x) => return ParseResult::Ok(Self::Expression(x)),
            _ => {}
        }
        match Eol::try_parse(token_iter) {
            ParseResult::Ok(_) => return ParseResult::Ok(Self::Eol),
            _ => {}
        }
        return ParseResult::Err(vec![ParseError::indexed(
            "אסימון לא ידוע",
            token_iter.index
        )]);
    }
}

pub fn parse(code: &[Token]) -> ParseResult<Vec<Node>> {
    let mut nodes: Vec<Node> = vec![];
    let mut token_iter: TokenIter = TokenIter::new(code);
    while token_iter.has_next() {
        nodes.push(huh!(Node::try_parse(&mut token_iter)));
    }
    ParseResult::Ok(nodes)
}