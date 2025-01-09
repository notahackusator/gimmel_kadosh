use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::lexer::prelude::*;
use crate::parser::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub base_values: Box<[BaseValue]>,
    pub operators: Box<[Operator]>
}

impl Parseable for Expression {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        let first = chain_err!(token_iter start BaseValue::try_parse(token_iter),
            err: ParseError::indexed("בתוך ביטוי", token_iter.index),
            skip: ParseError::indexed("ציפה לערך התחלתי עבור ביטוי", token_iter.index));
        let mut base_values = vec![first];
        let mut operators = vec![];

        while token_iter.has_next() {
            if Eol::try_parse(token_iter).is_ok() {
                token_iter.index -= 1;
                break;
            }

            let operator = chain_err!(token_iter start Operator::try_parse(token_iter),
                err: ParseError::indexed("ציפה לאופרטור", token_iter.index),
                skip: ParseError::indexed("ציפה לאופרטור", token_iter.index));
            let base_value = chain_err!(token_iter start BaseValue::try_parse(token_iter),
                err: ParseError::indexed("ציפה לערך", token_iter.index),
                skip: ParseError::indexed("ציפה לערך", token_iter.index));

            operators.push(operator);
            base_values.push(base_value);
        }

        ParseResult::Ok(Self {
            base_values: base_values.into(),
            operators: operators.into()
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BaseValue {
    Value {
        // יכול להיות לדוגמה (מספר) *או* (מערך שבתוכו אורך)
        value: Result<Path, Token>
    },
    FunctionCall {
        executable: Path,
        parameters: Box<[Expression]>
    },
    Constructor {
        structure: Path,
        parameters: Box<[Expression]>
    },
    Array {
        items: Box<[Expression]>
    },
    Index {
        array: Path,
        index: Box<Expression>
    }
}

impl Parseable for BaseValue {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        parse_start!(token_iter start);
        if !token_iter.has_next() {
            return ParseResult::Skip;
        }

        let comma = TokenRule::Divider(",".to_string());

        match try_parse_array(token_iter) {
            ParseResult::Ok(array) => return ParseResult::Ok(array),
            ParseResult::Err(err) => return ParseResult::Err(err),
            ParseResult::Skip => {}
        }

        let value_regex = TokenRegex::new(vec![
            TokenSequence::new(Some("value".to_string()), TokenRule::Value, vec![], None)
        ]);
        #[allow(suspicious_double_ref_op)]
        match value_regex.try_parse(token_iter) {
            ParseResult::Ok(value) => return ParseResult::Ok(Self::Value {
                value: Err(value.get("value").unwrap().clone().clone())
            }),
            _ => {}
        }

        let path = chain_err!(token_iter start Path::try_parse(token_iter),
            err: ParseError::new("בתוך ערך"));

        let constructor_regex = TokenRegex::new(vec![
            TokenSequence::new(None, TokenRule::UniqueId("חדש".to_string()), vec![], None)
        ]);
        if constructor_regex.try_parse(token_iter).is_ok() {
            return ParseResult::Ok(Self::Constructor {
                structure: path.clone(),
                parameters: huh!(parse_fn_call_params(token_iter)).into_boxed_slice()
            });
        }

        let parenthesis_or_square_bracket = TokenRegex::new(vec![
            TokenSequence::new(Some("p".to_string()), TokenRule::Divider("(".to_string()), vec![], None),
            TokenSequence::new(Some("s".to_string()), TokenRule::Divider("[".to_string()), vec![], None),
        ]);

        match parenthesis_or_square_bracket.try_parse(token_iter) {
            ParseResult::Ok(map) => {
                token_iter.index -= 1;
                match map.get("p") {
                    Some(_) => ParseResult::Ok(Self::FunctionCall {
                        executable: path.clone(),
                        parameters: huh!(parse_fn_call_params(token_iter)).into_boxed_slice()
                    }),
                    None => ParseResult::Ok(Self::Index {
                        array: path.clone(),
                        index: Box::new(huh!(parse_index(token_iter)))
                    })
                }
            }
            ParseResult::Skip => ParseResult::Ok(Self::Value {
                value: Ok(path.clone()),
            }),
            _ => unreachable!()
        }
    }
}

fn try_parse_array(token_iter: &mut TokenIter) -> ParseResult<BaseValue> {
    let comma = TokenRule::Divider(",".to_string());

    let square_brackets = TokenEnclosing::square_brackets();
    match square_brackets.try_parse(token_iter) {
        ParseResult::Ok(mut item_iter) => {
            let mut items = vec![];
            while item_iter.has_next() {
                match Expression::try_parse(&mut item_iter) {
                    ParseResult::Ok(expr) => items.push(expr),
                    ParseResult::Err(err) => return ParseResult::Err(err),
                    ParseResult::Skip => unreachable!()
                }

                if !item_iter.has_next() {
                    break;
                }

                let next = item_iter.next().unwrap();
                if !comma.matches(&next.token_type) {
                    return ParseResult::Err(vec![ParseError::indexed(
                        format!("ציפה לפסיק, מצא {}", next.token_type),
                        item_iter.index - 1
                    )]);
                }
            }
            return ParseResult::Ok(BaseValue::Array {
                items: items.into_boxed_slice()
            });
        },
        ParseResult::Err(errors) => {
            let err = &errors[0];
            if err.get_reason() != &square_brackets.open_fail {
                return ParseResult::Err(errors);
            }
        }
        ParseResult::Skip => unreachable!()
    }

    ParseResult::Skip
}

fn parse_index(token_iter: &mut TokenIter) -> ParseResult<Expression> {
    let square_brackets = TokenEnclosing::square_brackets();
    let mut index_iter = match square_brackets.try_parse(token_iter) {
        ParseResult::Ok(index_iter) => index_iter,
        ParseResult::Err(errors) => return ParseResult::Err(errors),
        ParseResult::Skip => unreachable!()
    };

    let index = match Expression::try_parse(&mut index_iter) {
        ParseResult::Ok(index) => index,
        ParseResult::Err(err) => return ParseResult::Err(err),
        ParseResult::Skip => unreachable!()
    };

    if index_iter.has_next() {
        return ParseResult::Err(vec![ParseError::indexed("לא ציפה לאסימון", index_iter.index)]);
    }

    ParseResult::Ok(index)
}

fn parse_fn_call_params(token_iter: &mut TokenIter) -> ParseResult<Vec<Expression>> {
    let comma = TokenRule::Divider(",".to_string());

    let parentheses = TokenEnclosing::parentheses();
    let mut param_iter = match parentheses.try_parse(token_iter) {
        ParseResult::Ok(params) => params,
        ParseResult::Err(errors) => return ParseResult::Err(errors),
        ParseResult::Skip => unreachable!()
    };

    let mut params = vec![];
    while param_iter.has_next() {
        match Expression::try_parse(&mut param_iter) {
            ParseResult::Ok(expr) => params.push(expr),
            ParseResult::Err(err) => return ParseResult::Err(err),
            ParseResult::Skip => unreachable!()
        }

        if !param_iter.has_next() {
            break;
        }

        let next = param_iter.next().unwrap();
        if !comma.matches(&next.token_type) {
            return ParseResult::Err(vec![ParseError::indexed(
                format!("ציפה לפסיק, מצא {}", next.token_type),
                param_iter.index - 1
            )]);
        }
    }

    ParseResult::Ok(params)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
    Ne
}

impl Parseable for Operator {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        let token = match token_iter.next() {
            None => return ParseResult::Err(vec![ParseError::indexed(
                "ציפה לאופרטור, אך לא נמצאו אסימונים", token_iter.index)]),
            Some(token) => token
        };

        if let TokenType::Identifier(div) = token.token_type {
            match div.as_str() {
                "ועוד" => ParseResult::Ok(Self::Add),
                "פחות" => ParseResult::Ok(Self::Sub),
                "כפול" => ParseResult::Ok(Self::Mul),
                "חלקי" => ParseResult::Ok(Self::Div),
                "שארית" => ParseResult::Ok(Self::Mod),

                "קטן_מ" => ParseResult::Ok(Self::Lt),
                "לכל_היותר" => ParseResult::Ok(Self::Le),
                "שווה_ל" => ParseResult::Ok(Self::Eq),
                "לפחות" => ParseResult::Ok(Self::Ge),
                "גדול_מ" => ParseResult::Ok(Self::Gt),

                "אינו" => ParseResult::Ok(Self::Ne),
                _ => {
                    token_iter.index -= 1;
                    ParseResult::Skip
                }
            }
        } else {
            token_iter.index -= 1;
            ParseResult::Skip
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub path: Box<[Token]>
}

impl Parseable for Path {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        let mut path = vec![];
        let start = TokenRegex::new(vec![TokenSequence::new(
            Some("start".to_string()), TokenRule::Id, vec![], Some("ציפה לשם".to_string())
        )]);
        #[allow(suspicious_double_ref_op)]
        path.push(huh!(start.try_parse(token_iter)).get("start").unwrap().clone().clone());
        let continuation = TokenRegex::new(vec![TokenSequence::new(
            None, TokenRule::UniqueId("שבתוכו".to_string()), vec![TokenSequence::new(
                Some("token".to_string()), TokenRule::Id, vec![

                ], Some("ציפה לשם".to_string())
            )], None
        )]);
        #[allow(suspicious_double_ref_op)]
        while token_iter.has_next() {
            match continuation.try_parse(token_iter) {
                ParseResult::Ok(map) => path.push(map.get("token").unwrap().clone().clone()),
                ParseResult::Err(err) => return ParseResult::Err(err),
                ParseResult::Skip => break
            }
        }
        ParseResult::Ok(Self {
            path: path.into_boxed_slice()
        })
    }
}

macro_rules! new_path {
    ($($token:expr),*$(,)?) => {
        crate::parser::prelude::Path {
            path: vec![$($token),*].into_boxed_slice()
        }
    };
}

pub(crate) use new_path;