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