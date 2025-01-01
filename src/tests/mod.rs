#[cfg(test)]
mod lexer {
    use crate::lexer::prelude::*;

    #[test]
    pub fn comments() {
        let tokens = interpret(
"
//קומנט
123
//456
",
            &Regexes::default()
        );
        let mut iter = tokens.into_iter();
        assert_eq!(iter.next(), Some(Token::new(TokenType::Integer("123".into()), 3, 1)));
    }

    #[test]
    pub fn numbers() {
        let tokens = interpret("123 456 789", &Regexes::default());
        let mut iter = tokens.into_iter();
        assert_eq!(iter.next(), Some(Token::new(TokenType::Integer("123".into()), 1, 1)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::Integer("456".into()), 1, 5)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::Integer("789".into()), 1, 9)));

        let tokens = interpret("0x123 3.14 0b1010", &Regexes::default());
        let mut iter = tokens.into_iter();
        assert_eq!(iter.next(), Some(Token::new(TokenType::Integer("0x123".into()), 1, 1)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::Decimal("3".into(), "14".into()), 1, 7)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::Integer("0b1010".into()), 1, 12)));
    }

    #[test]
    pub fn strings() {
        let tokens = interpret(
r#"
"שלום, עולם!"
"גרשיים בתוך גרשיים -> \" <- גרשיים בתוך גרשיים!"
"בקסלש -> \\ <- בקסלש"
"#,
            &Regexes::default()
        );

        let mut iter = tokens.into_iter();
        assert_eq!(iter.next(), Some(Token::new(TokenType::String(r#""שלום, עולם!""#.into()), 2, 1)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::String(r#""גרשיים בתוך גרשיים -> \" <- גרשיים בתוך גרשיים!""#.into()), 3, 1)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::String(r#""בקסלש -> \\ <- בקסלש""#.into()), 4, 1)));
    }

    #[test]
    pub fn identifiers() {
        let tokens = interpret(
r#"
שלום, מה שלומך?
"#, &Regexes::default());

        let mut iter = tokens.into_iter();
        assert_eq!(iter.next(), Some(Token::new(TokenType::Identifier("שלום".into()), 2, 1)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::Divider(",".into()), 2, 5)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::Identifier("מה".into()), 2, 7)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::Identifier("שלומך".into()), 2, 10)));
        assert_eq!(iter.next(), Some(Token::new(TokenType::Divider("?".into()), 2, 15)));
    }
}

#[cfg(test)]
mod parser {
    use crate::lexer::prelude::{interpret, Regexes};
    use crate::parser::prelude::{ParseError, TokenIter, TokenRegex, TokenRule, TokenSequence};

    #[test]
    pub fn token_regex() {
        // This rule expects identifier 'hello' followed by an identifier as a name
        let regex = TokenRegex::new(vec![
            TokenSequence::new(None, TokenRule::UniqueId("hello".to_string()), Some(Box::new(
                TokenSequence::new(Some("name".to_string()), TokenRule::Id, None, Some("Expected name after hello".to_string()))
            )), None)
        ]);

        let tokens = interpret("hello world", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        let parsing = regex.try_parse(&mut token_iter);
        assert!(parsing.is_ok());
        let parsing = parsing.unwrap();
        assert_eq!(parsing.get("name"), Some(&&tokens[1]));

        let tokens = interpret("hello 1", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        let parsing = regex.try_parse(&mut token_iter);
        assert!(parsing.is_err());
        let errors = parsing.unwrap_err();
        assert_eq!(errors, vec![ParseError::Indexed {
            reason: "Expected name after hello, found 1".to_string(),
            index: 1,
        }]);
    }
}

/*
#[cfg(test)]
mod parser {
    use crate::{lexer::prelude::{interpret, Regexes}, parser::prelude::parse};
    use crate::lexer::prelude::{Token, TokenType};
    use crate::parser::prelude::{BaseValue, Expression, Node, Operator, ParseResult, Var};

    #[test]
    pub fn variables() {
        // todo: end of function call is broken for some reason
        // todo: it might've returned skip in expression?
        // it might be continuing to parse the same variable 3...
        let tokens = interpret(
r#"
ויהי משתנה ושמו חמש וערכו 5 :
ויהי משתנה ושמו חמש_ועוד_אחד וערכו 5 ועוד 1 :
יהי שם_משתנה וערכו תוצאות טקס השם_טקס ; חמש, חמש_ועוד_אחד :
הגדר משתנה ששמו אחת וערכו 1 :
//הגדר בבשקה משתנה ששמו שתיים וערכו 2 :
"#, &Regexes::default());
        let parse = parse(&tokens);
        assert_eq!(parse, ParseResult::Ok(vec![Node::Var(Var {
            name: Token {
                token_type: TokenType::Identifier(
                    "חמש".to_string(),
                ),
                line: 2,
                col: 17,
            },
            expression: Expression {
                base_values: Box::new([
                    BaseValue::Value {
                        token: Token {
                            token_type: TokenType::Integer(
                                "5".into(),
                            ),
                            line: 2,
                            col: 27,
                        },
                    },
                ]),
                operators: Box::new([]),
            },
        }), Node::Var(Var {
            name: Token {
                token_type: TokenType::Identifier(
                    "חמש_ועוד_אחד".to_string(),
                ),
                line: 3,
                col: 17,
            },
            expression: Expression {
                base_values: Box::new([
                    BaseValue::Value {
                        token: Token {
                            token_type: TokenType::Integer(
                                "5".into(),
                            ),
                            line: 3,
                            col: 36,
                        },
                    },
                    BaseValue::Value {
                        token: Token {
                            token_type: TokenType::Integer(
                                "1".into(),
                            ),
                            line: 3,
                            col: 43,
                        },
                    },
                ]),
                operators: Box::new([
                    Operator::Add
                ]),
            },
        }), Node::Var(Var {
            name: Token {
                token_type: TokenType::Identifier(
                    "שם_משתנה".to_string(),
                ),
                line: 4,
                col: 5,
            },
            expression: Expression {
                base_values: Box::new([
                    BaseValue::FunctionCall {
                        name: Token {
                            token_type: TokenType::Identifier(
                                "שם_טקס".into()
                            ),
                            line: 4,
                            col: 31
                        },
                        parameters: Box::new([
                            Expression {
                                base_values: Box::new([
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Identifier(
                                                "חמש".into()
                                            ),
                                            line: 4,
                                            col: 41
                                        },
                                    }
                                ]),
                                operators: Box::new([]),
                            },
                            Expression {
                                base_values: Box::new([
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Identifier(
                                                "חמש_ועוד_אחד".into()
                                            ),
                                            line: 4,
                                            col: 46
                                        },
                                    }
                                ]),
                                operators: Box::new([]),
                            }
                        ]),
                    },
                ]),
                operators: Box::new([]),
            },
        })]))
    }
}*/