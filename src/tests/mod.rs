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


        let tokens = interpret(r#"1 ועוד "2" פחות 4 כפול פעולה(2 חלקי 3, 4 שארית 2)"#, &Regexes::default());
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
                            token_type: TokenType::String(
                                "\"2\"".to_string(),
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
                            col: 17,
                        },
                    },
                    BaseValue::FunctionCall {
                        name: Token {
                            token_type: TokenType::Identifier(
                                "פעולה".to_string(),
                            ),
                            line: 1,
                            col: 24,
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
                                            col: 30,
                                        },
                                    },
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Integer(
                                                "3".to_string(),
                                            ),
                                            line: 1,
                                            col: 37,
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
                                            col: 40,
                                        },
                                    },
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Integer(
                                                "2".to_string(),
                                            ),
                                            line: 1,
                                            col: 48,
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
    pub fn expression_errors() {
        let tokens = interpret("אחד ועוד ", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        let expression = Expression::try_parse(&mut token_iter);

        assert_eq!(expression, ParseResult::Err(vec![
            ParseError::indexed("ציפה לערך", 2)
        ]));

        let tokens = interpret("אחד ועודמילהשאינהאופרטור", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        let expression = Expression::try_parse(&mut token_iter);

        assert_eq!(expression, ParseResult::Err(vec![
            ParseError::indexed("ציפה לאופרטור", 1)
        ]));
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

    #[test]
    pub fn variable_errors() {
        let tokens = interpret("ויהי משתנה ושמו אחד וערכו", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        let variable = Var::try_parse(&mut token_iter);

        assert_eq!(variable, ParseResult::Err(vec![
            ParseError::indexed("ציפה לערך התחלתי עבור ביטוי", 5),
            ParseError::new("בתוך משתנה")
        ]));

        let tokens = interpret("ויהי משתנה ושמו אלף", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        let variable = Var::try_parse(&mut token_iter);

        assert_eq!(variable, ParseResult::Err(vec![
            ParseError::indexed("ציפה למילה 'וערכו', אך לא נמצאו אסימונים", 4),
            ParseError::new("בתוך משתנה")
        ]));

        let tokens = interpret("ויהי משתנה ושמו", &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        let variable = Var::try_parse(&mut token_iter);

        assert_eq!(variable, ParseResult::Err(vec![
            ParseError::indexed("המשתנה ציפה לשם, אך לא נמצאו אסימונים", 3),
            ParseError::new("בתוך משתנה")
        ]));
    }

    #[test]
    pub fn conditionals() {
        let tokens = interpret(r#"
        אם 1 ועוד 1 שווה_ל 2 (
            ויאמר("1 ועוד 1 שווה ל 2"):
        )
        ואם לא אך 3 פחות 3 שווה_ל 0 (
            ויאמר("3 פחות 3 שווה ל 0"):
        )
        ואם עדיין לא אך 2 חלקי 2 שווה_ל 1 (
            ויאמר("2 חלקי 2 שווה ל 1"):
        )
        ואם אפילו עדיין לא (
            ויאמר("אני לא יודע חשבון"):
        )
        "#, &Regexes::default());
        // ^-- It might look like the if statements have 2 opening parentheses, but it's actually
        // one open and one close. This is because open/close parentheses change direction depending
        // on whether you're writing in RTL / LTR. Your IDE will probably think the final
        // parenthesis is written it LTR instead of RTL and therefore format it for LTR.

        let nodes = parse(&tokens);

        assert_eq!(nodes, ParseResult::Ok(
            vec![
                Node::If(
                    If {
                        condition: Expression {
                            base_values: Box::new([
                                BaseValue::Value {
                                    token: Token {
                                        token_type: TokenType::Integer(
                                            "1".to_string(),
                                        ),
                                        line: 2,
                                        col: 12,
                                    },
                                },
                                BaseValue::Value {
                                    token: Token {
                                        token_type: TokenType::Integer(
                                            "1".to_string(),
                                        ),
                                        line: 2,
                                        col: 19,
                                    },
                                },
                                BaseValue::Value {
                                    token: Token {
                                        token_type: TokenType::Integer(
                                            "2".to_string(),
                                        ),
                                        line: 2,
                                        col: 28,
                                    },
                                },
                            ]),
                            operators: Box::new([
                                Operator::Add,
                                Operator::Eq,
                            ]),
                        },
                        code: vec![
                            Node::Expression(
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::FunctionCall {
                                            name: Token {
                                                token_type: TokenType::Identifier(
                                                    "ויאמר".to_string(),
                                                ),
                                                line: 3,
                                                col: 13,
                                            },
                                            parameters: Box::new([
                                                Expression {
                                                    base_values: Box::new([
                                                        BaseValue::Value {
                                                            token: Token {
                                                                token_type: TokenType::String(
                                                                    "\"1 ועוד 1 שווה ל 2\"".to_string(),
                                                                ),
                                                                line: 3,
                                                                col: 19,
                                                            },
                                                        },
                                                    ]),
                                                    operators: Box::new([]),
                                                },
                                            ]),
                                        },
                                    ]),
                                    operators: Box::new([]),
                                },
                            ),
                            Node::Eol,
                        ],
                    },
                ),
                Node::ElseIf(
                    ElseIf {
                        level: 0,
                        condition: Expression {
                            base_values: Box::new([
                                BaseValue::Value {
                                    token: Token {
                                        token_type: TokenType::Integer(
                                            "3".to_string(),
                                        ),
                                        line: 5,
                                        col: 19,
                                    },
                                },
                                BaseValue::Value {
                                    token: Token {
                                        token_type: TokenType::Integer(
                                            "3".to_string(),
                                        ),
                                        line: 5,
                                        col: 26,
                                    },
                                },
                                BaseValue::Value {
                                    token: Token {
                                        token_type: TokenType::Integer(
                                            "0".to_string(),
                                        ),
                                        line: 5,
                                        col: 35,
                                    },
                                },
                            ]),
                            operators: Box::new([
                                Operator::Sub,
                                Operator::Eq,
                            ]),
                        },
                        code: vec![
                            Node::Expression(
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::FunctionCall {
                                            name: Token {
                                                token_type: TokenType::Identifier(
                                                    "ויאמר".to_string(),
                                                ),
                                                line: 6,
                                                col: 13,
                                            },
                                            parameters: Box::new([
                                                Expression {
                                                    base_values: Box::new([
                                                        BaseValue::Value {
                                                            token: Token {
                                                                token_type: TokenType::String(
                                                                    "\"3 פחות 3 שווה ל 0\"".to_string(),
                                                                ),
                                                                line: 6,
                                                                col: 19,
                                                            },
                                                        },
                                                    ]),
                                                    operators: Box::new([]),
                                                },
                                            ]),
                                        },
                                    ]),
                                    operators: Box::new([]),
                                },
                            ),
                            Node::Eol,
                        ],
                    },
                ),
                Node::ElseIf(
                    ElseIf {
                        level: 2,
                        condition: Expression {
                            base_values: Box::new([
                                BaseValue::Value {
                                    token: Token {
                                        token_type: TokenType::Integer(
                                            "2".to_string(),
                                        ),
                                        line: 8,
                                        col: 25,
                                    },
                                },
                                BaseValue::Value {
                                    token: Token {
                                        token_type: TokenType::Integer(
                                            "2".to_string(),
                                        ),
                                        line: 8,
                                        col: 32,
                                    },
                                },
                                BaseValue::Value {
                                    token: Token {
                                        token_type: TokenType::Integer(
                                            "1".to_string(),
                                        ),
                                        line: 8,
                                        col: 41,
                                    },
                                },
                            ]),
                            operators: Box::new([
                                Operator::Div,
                                Operator::Eq,
                            ]),
                        },
                        code: vec![
                            Node::Expression(
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::FunctionCall {
                                            name: Token {
                                                token_type: TokenType::Identifier(
                                                    "ויאמר".to_string(),
                                                ),
                                                line: 9,
                                                col: 13,
                                            },
                                            parameters: Box::new([
                                                Expression {
                                                    base_values: Box::new([
                                                        BaseValue::Value {
                                                            token: Token {
                                                                token_type: TokenType::String(
                                                                    "\"2 חלקי 2 שווה ל 1\"".to_string(),
                                                                ),
                                                                line: 9,
                                                                col: 19,
                                                            },
                                                        },
                                                    ]),
                                                    operators: Box::new([]),
                                                },
                                            ]),
                                        },
                                    ]),
                                    operators: Box::new([]),
                                },
                            ),
                            Node::Eol,
                        ],
                    },
                ),
                Node::Else(
                    Else {
                        level: 3,
                        code: vec![
                            Node::Expression(
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::FunctionCall {
                                            name: Token {
                                                token_type: TokenType::Identifier(
                                                    "ויאמר".to_string(),
                                                ),
                                                line: 12,
                                                col: 13,
                                            },
                                            parameters: Box::new([
                                                Expression {
                                                    base_values: Box::new([
                                                        BaseValue::Value {
                                                            token: Token {
                                                                token_type: TokenType::String(
                                                                    "\"אני לא יודע חשבון\"".to_string(),
                                                                ),
                                                                line: 12,
                                                                col: 19,
                                                            },
                                                        },
                                                    ]),
                                                    operators: Box::new([]),
                                                },
                                            ]),
                                        },
                                    ]),
                                    operators: Box::new([]),
                                },
                            ),
                            Node::Eol,
                        ],
                    },
                ),
            ],
        ));
    }

    #[test]
    pub fn functions() {
        let tokens = interpret(r#"
        ויהי פעולה ושמה הדפס_שלום הלוקחת (שם) (
            ויאמר("שלום " ועוד שם ועוד "!"):
        )
        "#, &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        assert_eq!(Function::try_parse(&mut token_iter), ParseResult::Ok(
            Function {
                name: Token {
                    token_type: TokenType::Identifier(
                        "הדפס_שלום".to_string(),
                    ),
                    line: 2,
                    col: 25,
                },
                params: vec![
                    Token {
                        token_type: TokenType::Identifier(
                            "שם".to_string()
                        ),
                        line: 2,
                        col: 43
                    }
                ],
                code: vec![
                    Node::Expression(
                        Expression {
                            base_values: Box::new([
                                BaseValue::FunctionCall {
                                    name: Token {
                                        token_type: TokenType::Identifier(
                                            "ויאמר".to_string(),
                                        ),
                                        line: 3,
                                        col: 13,
                                    },
                                    parameters: Box::new([
                                        Expression {
                                            base_values: Box::new([
                                                BaseValue::Value {
                                                    token: Token {
                                                        token_type: TokenType::String(
                                                            "\"שלום \"".to_string(),
                                                        ),
                                                        line: 3,
                                                        col: 19,
                                                    },
                                                },
                                                BaseValue::Value {
                                                    token: Token {
                                                        token_type: TokenType::Identifier(
                                                            "שם".to_string(),
                                                        ),
                                                        line: 3,
                                                        col: 32,
                                                    },
                                                },
                                                BaseValue::Value {
                                                    token: Token {
                                                        token_type: TokenType::String(
                                                            "\"!\"".to_string(),
                                                        ),
                                                        line: 3,
                                                        col: 40,
                                                    },
                                                },
                                            ]),
                                            operators: Box::new([
                                                Operator::Add,
                                                Operator::Add
                                            ]),
                                        },
                                    ]),
                                },
                            ]),
                            operators: Box::new([]),
                        },
                    ),
                    Node::Eol,
                ],
            },
        ));
    }

    #[test]
    pub fn structs() {
        let tokens = interpret(r#"
        ויהי אוסף תכונות ושמו בן_אדם (
            שם_פרטי
            שם_משפחה
            תאריך_יום_הולדת
        )

        יהי משהו וערכו בן_אדם חדש ("בן", "אדם", "1/1/1970"):
        "#, &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        assert_eq!(Struct::try_parse(&mut token_iter), ParseResult::Ok(
            Struct {
                name: Token {
                    token_type: TokenType::Identifier(
                        "בן_אדם".to_string(),
                    ),
                    line: 2,
                    col: 31,
                },
                fields: vec![
                    Token {
                        token_type: TokenType::Identifier(
                            "שם_פרטי".to_string(),
                        ),
                        line: 3,
                        col: 13,
                    },
                    Token {
                        token_type: TokenType::Identifier(
                            "שם_משפחה".to_string(),
                        ),
                        line: 4,
                        col: 13,
                    },
                    Token {
                        token_type: TokenType::Identifier(
                            "תאריך_יום_הולדת".to_string(),
                        ),
                        line: 5,
                        col: 13,
                    },
                ],
            },
        ));

        assert_eq!(Var::try_parse(&mut token_iter), ParseResult::Ok(
            Var {
                name: Token {
                    token_type: TokenType::Identifier(
                        "משהו".to_string(),
                    ),
                    line: 8,
                    col: 13,
                },
                value: Expression {
                    base_values: Box::new([
                        BaseValue::Constructor {
                            structure: Token {
                                token_type: TokenType::Identifier(
                                    "בן_אדם".to_string(),
                                ),
                                line: 8,
                                col: 24,
                            },
                            parameters: Box::new([
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::Value {
                                            token: Token {
                                                token_type: TokenType::String(
                                                    "\"בן\"".to_string(),
                                                ),
                                                line: 8,
                                                col: 36,
                                            },
                                        },
                                    ]),
                                    operators: Box::new([]),
                                },
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::Value {
                                            token: Token {
                                                token_type: TokenType::String(
                                                    "\"אדם\"".to_string(),
                                                ),
                                                line: 8,
                                                col: 42,
                                            },
                                        },
                                    ]),
                                    operators: Box::new([]),
                                },
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::Value {
                                            token: Token {
                                                token_type: TokenType::String(
                                                    "\"1/1/1970\"".to_string(),
                                                ),
                                                line: 8,
                                                col: 49,
                                            },
                                        },
                                    ]),
                                    operators: Box::new([]),
                                },
                            ]),
                        },
                    ]),
                    operators: Box::new([]),
                },
            },
        ));
    }

    #[test]
    pub fn changes() {
        let tokens = interpret(r#"
        שנה את אחד להיות 2:
        שנה את משהו להיות בן_אדם חדש(
            "מא",
            "בולז",
            "לול"
        ):
        "#, &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);
        assert_eq!(Change::try_parse(&mut token_iter), ParseResult::Ok(
            Change {
                name: Token {
                    token_type: TokenType::Identifier(
                        "אחד".to_string(),
                    ),
                    line: 2,
                    col: 16,
                },
                value: Expression {
                    base_values: Box::new([
                        BaseValue::Value {
                            token: Token {
                                token_type: TokenType::Integer(
                                    "2".to_string(),
                                ),
                                line: 2,
                                col: 26,
                            },
                        },
                    ]),
                    operators: Box::new([]),
                },
            }
        ));
        while Eol::try_parse(&mut token_iter).is_ok() {}
        assert_eq!(Change::try_parse(&mut token_iter), ParseResult::Ok(
            Change {
                name: Token {
                    token_type: TokenType::Identifier(
                        "משהו".to_string(),
                    ),
                    line: 3,
                    col: 16,
                },
                value: Expression {
                    base_values: Box::new([
                        BaseValue::Constructor {
                            structure: Token {
                                token_type: TokenType::Identifier(
                                    "בן_אדם".to_string()
                                ),
                                line: 3,
                                col: 27,
                            },
                            parameters: Box::new([
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::Value {
                                            token: Token {
                                                token_type: TokenType::String("\"מא\"".to_string()),
                                                line: 4,
                                                col: 13
                                            }
                                        }
                                    ]),
                                    operators: Box::new([])
                                },
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::Value {
                                            token: Token {
                                                token_type: TokenType::String("\"בולז\"".to_string()),
                                                line: 5,
                                                col: 13
                                            }
                                        }
                                    ]),
                                    operators: Box::new([])
                                },
                                Expression {
                                    base_values: Box::new([
                                        BaseValue::Value {
                                            token: Token {
                                                token_type: TokenType::String("\"לול\"".to_string()),
                                                line: 6,
                                                col: 13
                                            }
                                        }
                                    ]),
                                    operators: Box::new([])
                                },
                            ]),
                        }
                    ]),
                    operators: Box::new([]),
                },
            }
        ));
    }

    #[test]
    pub fn loops() {
        let tokens = interpret(r#"
        ויהי משתנה ושמו אינדקס וערכו 0:
        כל עוד אינדקס קטן_מ 10 (
            אם אינדקס שווה_ל 3 (
                דלג
            )
            ואם לא אך אינדקס שווה_ל 6 (
                עצור
            )
            ויאמר(אינדקס):
            שנה את אינדקס להיות אינדקס ועוד 1:
        )
        "#, &Regexes::default());
        let mut token_iter = TokenIter::new(&tokens);

        assert!(Var::try_parse(&mut token_iter).is_ok());
        while Eol::try_parse(&mut token_iter).is_ok() {}
        assert_eq!(While::try_parse(&mut token_iter), ParseResult::Ok(
            While {
                condition: Expression {
                    base_values: Box::new([
                        BaseValue::Value {
                            token: Token {
                                token_type: TokenType::Identifier(
                                    "אינדקס".to_string(),
                                ),
                                line: 3,
                                col: 16,
                            },
                        },
                        BaseValue::Value {
                            token: Token {
                                token_type: TokenType::Integer(
                                    "10".to_string(),
                                ),
                                line: 3,
                                col: 29,
                            },
                        },
                    ]),
                    operators: Box::new([
                        Operator::Lt,
                    ]),
                },
                code: vec![
                    Node::If(
                        If {
                            condition: Expression {
                                base_values: Box::new([
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Identifier(
                                                "אינדקס".to_string()
                                            ),
                                            line: 4,
                                            col: 16
                                        }
                                    },
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Integer(
                                                "3".to_string()
                                            ),
                                            line: 4,
                                            col: 30
                                        }
                                    }
                                ]),
                                operators: Box::new([
                                    Operator::Eq
                                ])
                            },
                            code: vec![
                                Node::Expression(
                                    Expression {
                                        base_values: Box::new([
                                            BaseValue::Value {
                                                token: Token {
                                                    token_type: TokenType::Identifier(
                                                        "דלג".to_string()
                                                    ),
                                                    line: 5,
                                                    col: 17
                                                }
                                            }
                                        ]),
                                        operators: Box::new([

                                        ])
                                    }
                                )
                            ]
                        }
                    ),
                    Node::ElseIf(
                        ElseIf {
                            level: 0,
                            condition: Expression {
                                base_values: Box::new([
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Identifier(
                                                "אינדקס".to_string()
                                            ),
                                            line: 7,
                                            col: 23
                                        }
                                    },
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Integer(
                                                "6".to_string()
                                            ),
                                            line: 7,
                                            col: 37
                                        }
                                    }
                                ]),
                                operators: Box::new([
                                    Operator::Eq
                                ])
                            },
                            code: vec![
                                Node::Expression(
                                    Expression {
                                        base_values: Box::new([
                                            BaseValue::Value {
                                                token: Token {
                                                    token_type: TokenType::Identifier(
                                                        "עצור".to_string()
                                                    ),
                                                    line: 8,
                                                    col: 17
                                                }
                                            }
                                        ]),
                                        operators: Box::new([

                                        ])
                                    }
                                )
                            ]
                        }
                    ),
                    Node::Expression(
                        Expression {
                            base_values: Box::new([
                                BaseValue::FunctionCall {
                                    name: Token {
                                        token_type: TokenType::Identifier(
                                            "ויאמר".to_string(),
                                        ),
                                        line: 10,
                                        col: 13,
                                    },
                                    parameters: Box::new([
                                        Expression {
                                            base_values: Box::new([
                                                BaseValue::Value {
                                                    token: Token {
                                                        token_type: TokenType::Identifier(
                                                            "אינדקס".to_string(),
                                                        ),
                                                        line: 10,
                                                        col: 19,
                                                    },
                                                },
                                            ]),
                                            operators: Box::new([]),
                                        },
                                    ]),
                                },
                            ]),
                            operators: Box::new([]),
                        },
                    ),
                    Node::Eol,
                    Node::Change(
                        Change {
                            name: Token {
                                token_type: TokenType::Identifier(
                                    "אינדקס".to_string(),
                                ),
                                line: 11,
                                col: 20,
                            },
                            value: Expression {
                                base_values: Box::new([
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Identifier(
                                                "אינדקס".to_string(),
                                            ),
                                            line: 11,
                                            col: 33,
                                        },
                                    },
                                    BaseValue::Value {
                                        token: Token {
                                            token_type: TokenType::Integer(
                                                "1".to_string(),
                                            ),
                                            line: 11,
                                            col: 45,
                                        },
                                    },
                                ]),
                                operators: Box::new([
                                    Operator::Add,
                                ]),
                            },
                        },
                    ),
                    Node::Eol,
                ],
            },
        ));
    }
}