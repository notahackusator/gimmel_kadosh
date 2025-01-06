use crate::lexer::prelude::*;
use crate::parser::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Struct {
    pub name: Token,
    pub fields: Vec<Token>
}

impl Parseable for Struct {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let var_start_regex = TokenRegex::new(vec![
            TokenSequence::new(None, TokenRule::UniqueId("ויהי".to_string()), vec![
                TokenSequence::new(None, TokenRule::UniqueId("אוסף".to_string()), vec![
                    TokenSequence::new(None, TokenRule::UniqueId("תכונות".to_string()), vec![
                        TokenSequence::new(None, TokenRule::UniqueId("ושמו".to_string()), vec![
                            TokenSequence::new(Some("name".to_string()), TokenRule::Id, vec![

                            ], Some("אוסף התכונות ציפה לשם".to_string()))
                        ], Some("ציפה למילה 'ושמו'".to_string()))
                    ], Some("ציפה למילה 'תכונות'".to_string()))
                ], None)
            ], None),
        ]);

        let map = chain_err!(token_iter start var_start_regex.try_parse(token_iter),
            err: ParseError::new("בתוך אוסף תכונות"));
        #[allow(suspicious_double_ref_op)]
        let name = map.get("name").unwrap().clone().clone();

        let mut parentheses = chain_err!(token_iter start TokenEnclosing::parentheses().try_parse(token_iter),
            err: ParseError::new("בתוך אוסף תכונות"));
        let mut fields = vec![];

        let field = TokenRule::Id;
        while parentheses.has_next() {
            let token = parentheses.borrow_next().unwrap();
            if TokenRule::Id.matches(&token.token_type) {
                fields.push(token.clone());
            } else {
                token_iter.index = start;
                return ParseResult::Err(vec![ParseError::indexed("ציפה לשם של תכונה", parentheses.index - 1)]);
            }
        }

        ParseResult::Ok(Self {
            name,
            fields
        })
    }
}