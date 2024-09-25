use std::fmt::{Display, Formatter, Write};

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum TokenType {
    Identifier(Box<str>),
    Integer(Box<str>),
    Decimal(Box<str>, Box<str>),
    String(Box<str>),
    Char(Box<str>),
    Divider(Box<str>),
    Operator(Box<str>)
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Identifier(x) => f.pad(&x),
            TokenType::Integer(x) => f.pad(&x),
            TokenType::Decimal(a, b) => f.pad(&a).and(f.write_char('.')).and(f.pad(&b)),
            TokenType::String(x) => f.pad(&x),
            TokenType::Char(x) => f.pad(&x),
            TokenType::Divider(x) => f.pad(&x),
            TokenType::Operator(x) => f.pad(&x)
        }
    }
}

// can remove
impl TokenType {
    pub fn open_parentheses() -> Self {
        TokenType::Divider("(".into())
    }

    pub fn close_parentheses() -> Self {
        TokenType::Divider(")".into())
    }

    pub fn open_square_brackets() -> Self {
        TokenType::Divider("[".into())
    }

    pub fn close_square_brackets() -> Self {
        TokenType::Divider("]".into())
    }

    pub fn open_brackets() -> Self {
        TokenType::Divider("{".into())
    }

    pub fn close_brackets() -> Self {
        TokenType::Divider("}".into())
    }

    pub fn open_arrow_brackets() -> Self {
        TokenType::Divider("<".into())
    }

    pub fn close_arrow_brackets() -> Self {
        TokenType::Divider(">".into())
    }

    pub fn question_mark() -> Self {
        TokenType::Divider("?".into())
    }

    pub fn pipe() -> Self {
        TokenType::Operator("|".into())
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub col: usize
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, col: usize) -> Self {
        Self { token_type, line, col }
    }

    pub fn temp() -> Self {
        Self::new(TokenType::Identifier("".into()), 0, 0)
    }
}