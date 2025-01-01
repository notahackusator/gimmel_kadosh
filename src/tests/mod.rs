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
    use crate::lexer::prelude::*;
    use crate::parser::prelude::*;

    #[test]
    pub fn token_regex() {
        // This rule expects identifier 'hello' followed by an identifier as a name
        let regex = TokenRegex::new(vec![
            TokenSequence::new(None, TokenRule::UniqueId("hello".to_string()), vec![
                TokenSequence::new(Some("name".to_string()), TokenRule::Id, vec![], Some("Expected name after hello".to_string()))
            ], None)
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

    #[test]
    pub fn expression() {
        let tokens = interpret("אחד ועוד 2", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        let expression = Expression::try_parse(&mut token_iter);

        assert_eq!(expression, ParseResult::Ok(Expression {
            base_values: vec![
                BaseValue::Value {
                    token: Token {
                        token_type: TokenType::Identifier(
                            "אחד".to_string(),
                        ),
                        line: 1,
                        col: 1,
                    },
                },
                BaseValue::Value {
                    token: Token {
                        token_type: TokenType::Integer(
                            "2".to_string(),
                        ),
                        line: 1,
                        col: 10,
                    },
                },
            ].into(),
            operators: vec![
                Operator::Add
            ].into()
        }));


        let tokens = interpret("1 ועוד 2 פחות 4 כפול פעולה(2 חלקי 3, 4 שארית 2)", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        let expression = Expression::try_parse(&mut token_iter);

        assert_eq!(expression, ParseResult::Ok(
            Expression {
                base_values: Box::new([
                    BaseValue::Value {
                        token: Token {
                            token_type: TokenType::Integer(
                                "1".to_string(),
                            ),
                            line: 1,
                            col: 1,
                        },
                    },
                    BaseValue::Value {
                        token: Token {
                            token_type: TokenType::Integer(
                                "2".to_string(),
                            ),
                            line: 1,
                            col: 8,
                        },
                    },
                    BaseValue::Value {
                        token: Token {
                            token_type: TokenType::Integer(
                                "4".to_string(),
                            ),
                            line: 1,
                            col: 15,
                        },
                    },
                    BaseValue::FunctionCall {
                        name: Token {
                            token_type: TokenType::Identifier(
                                "פעולה".to_string(),
                            ),
                            line: 1,
                            col: 22,
                        },
                        parameters: Box::new([
                            Expression {
                                base_values: Box::new([
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Integer(
                                                "2".to_string(),
                                            ),
                                            line: 1,
                                            col: 28,
                                        },
                                    },
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Integer(
                                                "3".to_string(),
                                            ),
                                            line: 1,
                                            col: 35,
                                        },
                                    },
                                ]),
                                operators: Box::new([
                                    Operator::Div,
                                ]),
                            },
                            Expression {
                                base_values: Box::new([
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Integer(
                                                "4".to_string(),
                                            ),
                                            line: 1,
                                            col: 38,
                                        },
                                    },
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Integer(
                                                "2".to_string(),
                                            ),
                                            line: 1,
                                            col: 46,
                                        },
                                    },
                                ]),
                                operators: Box::new([
                                    Operator::Mod,
                                ]),
                            },
                        ]),
                    },
                ]),
                operators: Box::new([
                    Operator::Add,
                    Operator::Sub,
                    Operator::Mul,
                ]),
            },
        ));
    }

    #[test]
    pub fn variable() {
        let tokens = interpret("ויהי משתנה ושמו אחד וערכו 1:", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        assert_eq!(Var::try_parse(&mut token_iter), ParseResult::Ok(
            Var {
                name: Token {
                    token_type: TokenType::Identifier(
                        "אחד".to_string(),
                    ),
                    line: 1,
                    col: 17,
                },
                value: Expression {
                    base_values: Box::new([
                        BaseValue::Value {
                            token: Token {
                                token_type: TokenType::Integer(
                                    "1".to_string(),
                                ),
                                line: 1,
                                col: 27,
                            },
                        },
                    ]),
                    operators: Box::new([]),
                },
            },
        ));

        let tokens = interpret("יהי שלוש וערכו 1 ועוד 2:", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        assert_eq!(Var::try_parse(&mut token_iter), ParseResult::Ok(
            Var {
                name: Token {
                    token_type: TokenType::Identifier(
                        "שלוש".to_string(),
                    ),
                    line: 1,
                    col: 5,
                },
                value: Expression {
                    base_values: Box::new([
                        BaseValue::Value {
                            token: Token {
                                token_type: TokenType::Integer(
                                    "1".to_string(),
                                ),
                                line: 1,
                                col: 16,
                            },
                        },
                        BaseValue::Value {
                            token: Token {
                                token_type: TokenType::Integer(
                                    "2".to_string(),
                                ),
                                line: 1,
                                col: 23,
                            },
                        },
                    ]),
                    operators: Box::new([
                        Operator::Add
                    ]),
                },
            },
        ));
    }
}