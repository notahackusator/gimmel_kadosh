use std::cell::RefCell;
use std::rc::Rc;
use crate::condition;
use crate::lexer::prelude::Token;
use crate::parser::expression::Expression;
use crate::parser::token_iter::TokenIter;
use crate::parser::parseable::{Parseable, ParseResult};

use super::prelude::{Buf, ErrAction, Executor, Param};

#[derive(Clone, Debug, PartialEq)]
pub struct Var {
    pub name: Token,
    pub expression: Expression
}

impl Parseable for Var {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        dbg!(&token_iter.tokens[token_iter.index]);
        let name: Buf<Token> = Rc::new(RefCell::new(None));
        let expression: Buf<Expression> = Rc::new(RefCell::new(None));
        let debug: Buf<i32> = Rc::new(RefCell::new(None));
        let parse: ParseResult<()> = Executor::new()
            .cases(vec![
                Executor::new()
                    .id(Param::Value("ויהי"), ErrAction::Skip)
                    .id(Param::Value("משתנה"), ErrAction::Skip)
                    .id(Param::Value("ושמו"), ErrAction::Error)
                    .id(Param::Buf(name.clone()), ErrAction::Error)
                    .id(Param::Value("וערכו"), ErrAction::Error)
                    .node(expression.clone(), ErrAction::Error),
                Executor::new()
                    .id(Param::Value("יהי"), ErrAction::Skip)
                    .id(Param::Buf(name.clone()), ErrAction::Error)
                    .id(Param::Value("וערכו"), ErrAction::Error)
                    .condition({let debug = debug.clone(); Box::new(move |_| { debug.replace(Some(3)); ParseResult::Ok(()) })}, ErrAction::Error)
                    .condition(condition!(bufs: , |i| {
                        dbg!(&i);
                        ParseResult::Ok(())
                    }), ErrAction::Error)
                    .node(expression.clone(), ErrAction::Error)
                    .condition({let debug = debug.clone(); Box::new(move |_| { debug.replace(Some(4)); ParseResult::Ok(()) })}, ErrAction::Error),
                Executor::new()
                    .condition({let debug = debug.clone(); Box::new(move |_| { debug.replace(Some(1)); ParseResult::Ok(()) })}, ErrAction::Error)
                    .id(Param::Value("הגדר"), ErrAction::Skip)
                    .condition({let debug = debug.clone(); Box::new(move |_| { debug.replace(Some(2)); ParseResult::Ok(()) })}, ErrAction::Error)
                    .optional(Executor::new().id(Param::Value("בבקשה"), ErrAction::Error))
                    .condition({let debug = debug.clone(); Box::new(move |_| { debug.replace(Some(7)); ParseResult::Ok(()) })}, ErrAction::Error)
                    .id(Param::Value("משתנה"), ErrAction::Error)
                    .condition({let debug = debug.clone(); Box::new(move |_| { debug.replace(Some(8)); ParseResult::Ok(()) })}, ErrAction::Error)
                    .id(Param::Value("ששמו"), ErrAction::Error)
                    .id(Param::Buf(name.clone()), ErrAction::Error)
                    .id(Param::Value("וערכו"), ErrAction::Error)
                    .condition({let debug = debug.clone(); Box::new(move |_| { debug.replace(Some(5)); ParseResult::Ok(()) })}, ErrAction::Error)
                    .node(expression.clone(), ErrAction::Error)
                    .condition({let debug = debug.clone(); Box::new(move |_| { debug.replace(Some(6)); ParseResult::Ok(()) })}, ErrAction::Error)
            ], ErrAction::Error).execute(token_iter);
        dbg!(&name, &expression, &debug);
        match parse {
            ParseResult::Ok(_) => ParseResult::Ok(Self {
                name: name.take().unwrap(),
                expression: expression.take().unwrap()
            }),
            ParseResult::Err(vec) => ParseResult::Err(vec),
            ParseResult::Skip => ParseResult::Skip,
        }
    }
}