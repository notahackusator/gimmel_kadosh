use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use crate::condition;
use crate::lexer::prelude::{Token, TokenType};
use crate::parser::errors::ParseError;
use crate::parser::token_iter::TokenIter;
use crate::parser::parseable::{Parseable, ParseResult};

use super::prelude::{Buf, ErrAction, Executor, Param};

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub base_values: Box<[BaseValue]>,
    pub operators: Box<[Operator]>
}

impl Parseable for Expression {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        let first: Buf<BaseValue> = Rc::new(RefCell::new(None));
        let first_parse: ParseResult<()> = Executor::new()
            .node(first.clone(), ErrAction::Error)
            .execute(token_iter);
        match first_parse {
            ParseResult::Ok(_) => {},
            ParseResult::Err(err) => return ParseResult::Err(err),
            ParseResult::Skip => return ParseResult::Skip,
        }
        let mut base_values: Vec<BaseValue> = vec![first.take().unwrap()];
        let mut operators: Vec<Operator> = vec![];
        // dbg!(&base_values, &operators);
        if token_iter.has_next() {
            while let ParseResult::Err(_) = token_iter.divider(Param::Value(":"), ErrAction::Error) {
                let operator: Buf<Operator> = Rc::new(RefCell::new(None));
                let base_value: Buf<BaseValue> = Rc::new(RefCell::new(None));

                let parse: ParseResult<()> = Executor::new()
                    .node(operator.clone(), ErrAction::Error)
                    .node(base_value.clone(), ErrAction::Error)
                    .execute(token_iter);
                match parse {
                    ParseResult::Ok(_) => {},
                    ParseResult::Err(err) => return ParseResult::Err(err),
                    ParseResult::Skip => return ParseResult::Skip,
                }

                base_values.push(base_value.take().unwrap());
                operators.push(operator.take().unwrap());
            }
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
        token: Token
    },
    FunctionCall {
        name: Token,
        parameters: Box<[Expression]>
    }
}

impl Parseable for BaseValue {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        dbg!(token_iter.index, &token_iter.tokens[token_iter.index]);
        let function_name: Buf<Token> = Rc::new(RefCell::new(None));
        let function_params: Buf<Vec<Box<dyn Any>>> = Rc::new(RefCell::new(None));
        let value: Buf<Token> = Rc::new(RefCell::new(None));
        let parse: ParseResult<()> = Executor::new()
            .cases(vec![
                Executor::new()
                    .id(Param::Value("תוצאות"), ErrAction::Skip)
                    .id(Param::Value("טקס"), ErrAction::Error)
                    .id(Param::Buf(function_name.clone()), ErrAction::Error)
                    .condition(condition!(bufs: function_name, |index| {
                        let mut token: Token = function_name.clone().take().unwrap();
                        if let TokenType::Identifier(ref mut name) = &mut token.token_type {
                            if name.starts_with("ה") {
                                name.remove(0);
                                function_name.replace(Some(token));
                                ParseResult::Ok(())
                            } else {
                                ParseResult::Err(vec![
                                    ParseError::indexed("אי אפשר להתחיל טקס ללא ה' הידיעה", index)
                                ])
                            }
                        } else {
                            unreachable!()
                        }
                    }), ErrAction::Error)
                    .nodes::<Expression>(function_params.clone(), TokenType::Divider(",".into()), TokenType::Divider(";".into()), TokenType::Divider(":".into()), ErrAction::Error)
                    .condition(condition!(bufs: , |i| {dbg!("Good!", i); ParseResult::Ok(())}), ErrAction::Error),
                Executor::new()
                    .any(value.clone(), ErrAction::Error)
            ], ErrAction::Error)
            .execute(token_iter);
        dbg!(&function_name, &function_params, &value);
        match parse {
            ParseResult::Ok(_) => {},
            ParseResult::Err(err) => return ParseResult::Err(err),
            ParseResult::Skip => return ParseResult::Skip
        }
        if let Some(token) = value.take() {
            let value: Self = Self::Value { token };
            dbg!(&value);
            ParseResult::Ok(value)
        } else {
            let Some(name) = function_name.take() else { unreachable!() };
            let Some(params_any) = function_params.take() else { unreachable!() };
            let mut parameters: Vec<Expression> = vec![];
            for param_any in params_any {
                let cast_parameter: Expression = (&*param_any).downcast_ref::<Expression>().unwrap().clone();
                dbg!(&cast_parameter);
                parameters.push(cast_parameter);
            }
            let fn_call: Self = Self::FunctionCall { name, parameters: parameters.into() };
            dbg!(&fn_call);
            ParseResult::Ok(fn_call)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod
}

impl Parseable for Operator {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> {
        let this: Buf<Self> = Rc::new(RefCell::new(None));
        let parse: ParseResult<()> = Executor::new().cases(vec![
            Executor::new().id(Param::Value("ועוד"), ErrAction::Skip).condition({let this = this.clone(); Box::new(move |_| { this.replace(Some(Operator::Add)); ParseResult::Ok(()) }) }, ErrAction::Skip),
            Executor::new().id(Param::Value("פחות"), ErrAction::Skip).condition({let this = this.clone(); Box::new(move |_| { this.replace(Some(Operator::Sub)); ParseResult::Ok(()) }) }, ErrAction::Skip),
            Executor::new().id(Param::Value("כפול"), ErrAction::Skip).condition({let this = this.clone(); Box::new(move |_| { this.replace(Some(Operator::Mul)); ParseResult::Ok(()) }) }, ErrAction::Skip),
            Executor::new().id(Param::Value("חלקי"), ErrAction::Skip).condition({let this = this.clone(); Box::new(move |_| { this.replace(Some(Operator::Div)); ParseResult::Ok(()) }) }, ErrAction::Skip),
            Executor::new().id(Param::Value("שארית"), ErrAction::Skip).condition({let this = this.clone(); Box::new(move |_| { this.replace(Some(Operator::Mod)); ParseResult::Ok(()) }) }, ErrAction::Skip),
        ], ErrAction::Error).execute(token_iter);
        match parse {
            ParseResult::Ok(_) => {},
            ParseResult::Err(err) => return ParseResult::Err(err),
            ParseResult::Skip => return ParseResult::Skip
        }
        ParseResult::Ok(this.take().unwrap())
    }
}