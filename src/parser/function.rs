use crate::lexer::prelude::*;
use crate::parser::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub code: Vec<Node>
}

impl Parseable for Function {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let var_start_regex = TokenRegex::new(vec![
            TokenSequence::new(None, TokenRule::UniqueId("ויהי".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("פעולה".to_string()), vec![
                    TokenSequence::new(None, TokenRule::UniqueId("ושמה".to_string()), vec![
                        TokenSequence::new(Some("name".to_string()), TokenRule::Id, vec![
                            TokenSequence::new(None, TokenRule::UniqueId("הלוקחת".to_string()), vec![

                            ], Some("ציפה למילה 'הלוקחת'".to_string()))
                        ], Some("הפעולה ציפה לשם".to_string()))
                    ], Some("ציפה למילה 'ושמה'".to_string()))
                ], None)
            ], None),
        ]);

        let map = chain_err!(token_iter start var_start_regex.try_parse(token_iter),
            err: ParseError::new("בתוך פעולה"));
        #[allow(suspicious_double_ref_op)]
        let name = map.get("name").unwrap().clone().clone();

        let mut parentheses = chain_err!(token_iter start TokenEnclosing::parentheses().try_parse(token_iter),
            err: ParseError::new("בתוך פעולה"));
        let mut params = vec![];

        let param = TokenRule::Id;
        while parentheses.has_next() {
            let token = parentheses.borrow_next().unwrap();
            if TokenRule::Id.matches(&token.token_type) {
                params.push(token.clone());
            } else {
                token_iter.index = start;
                return ParseResult::Err(vec![ParseError::indexed("ציפה לשם של פרמטר", parentheses.index - 1)]);
            }
        }

        let mut code = vec![];

        let mut code_iter = chain_err!(token_iter start TokenEnclosing::parentheses().try_parse(token_iter),
            err: ParseError::new("בתוך פעולה"));
        while code_iter.has_next() {
            let node = Node::try_parse(&mut code_iter);

            code.push(chain_err!(token_iter start node,
                err: ParseError::new("בתוך פעולה")));
        }

        ParseResult::Ok(Self {
            name,
            params,
            code
        })
    }
}