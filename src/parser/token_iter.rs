use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::rc::Rc;

use crate::lexer::prelude::{Token, TokenType};
use crate::parser::errors::ParseError;
use crate::parser::parseable::{Parseable, ParseResult};

#[derive(Clone, PartialEq)]
pub struct TokenIter<'a> {
    pub tokens: &'a [Token],
    pub index: usize,
    pub start: Vec<usize>
}

impl<'a> TokenIter<'a> {
    pub fn new(tokens: &'a [Token]) -> TokenIter<'a> {
        TokenIter { tokens, index: 0, start: vec![] }
    }

    pub fn has_next(&self) -> bool {
        self.index < self.tokens.len()
    }
}

enum ExecFn<'a> {
    Id(Param<Token>, ErrAction),
    Divider(Param<Token>, ErrAction),
    Any(Buf<Token>, ErrAction),
    Optional(Executor<'a>),
    RepeatUntil(Buf<Vec<Vec<Token>>>, TokenType, TokenType, ErrAction),
    OpenClose(Buf<TokenIter<'a>>, TokenType, TokenType, ErrAction),
    Separate(Buf<Vec<TokenIter<'a>>>, TokenType, TokenType, TokenType, ErrAction),
    OpenCloseSeparate(Buf<Vec<TokenIter<'a>>>, TokenType, TokenType, TokenType, ErrAction),
    Nodes(Buf<Vec<Box<dyn Any>>>, Box<dyn Fn(&mut TokenIter) -> ParseResult<Box<dyn Any>>>, TokenType, TokenType, TokenType, ErrAction),
    Condition(Box<dyn FnOnce(usize) -> ParseResult<()>>, ErrAction),
    Node(Box<dyn FnOnce(&mut TokenIter) -> ParseResult<()>>, ErrAction),
    Cases(Vec<Executor<'a>>, ErrAction)
}

#[macro_export]
macro_rules! condition {
    (bufs: $($buf:ident),*, |$index:ident| $code:block) => {
        {
            $(let $buf = $buf.clone();)*
            Box::new(move |$index| $code)
        }
    };
}

pub struct Executor<'a> {
    functions: Vec<ExecFn<'a>>
}

impl<'a> Executor<'a> {
    pub fn new() -> Self {
        Self { functions: vec![] }
    }
}

impl<'a> Executor<'a> {
    pub fn id(mut self, param: Param<Token>, err_action: ErrAction) -> Self {
        self.functions.push(ExecFn::Id(param, err_action));
        self
    }

    pub fn divider(mut self, param: Param<Token>, err_action: ErrAction) -> Self {
        self.functions.push(ExecFn::Divider(param, err_action));
        self
    }

    pub fn any(mut self, buf: Buf<Token>, err_action: ErrAction) -> Self {
        self.functions.push(ExecFn::Any(buf, err_action));
        self
    }

    pub fn optional(mut self, executor: Executor<'a>) -> Self {
        self.functions.push(ExecFn::Optional(executor));
        self
    }

    pub fn open_close(mut self, buf: Buf<TokenIter<'a>>, open: TokenType, close: TokenType, err_action: ErrAction) -> Self {
        self.functions.push(ExecFn::OpenClose(buf, open, close, err_action));
        self
    }

    pub fn separate(mut self, buf: Buf<Vec<TokenIter<'a>>>, sep: TokenType, open: TokenType, close: TokenType, err_action: ErrAction) -> Self {
        self.functions.push(ExecFn::Separate(buf, sep, open, close, err_action));
        self
    }

    pub fn open_close_separate(mut self, buf: Buf<Vec<TokenIter<'a>>>, sep: TokenType, open: TokenType, close: TokenType, err_action: ErrAction) -> Self {
        self.functions.push(ExecFn::OpenCloseSeparate(buf, sep, open, close, err_action));
        self
    }

    pub fn nodes<P>(mut self, buf: Buf<Vec<Box<dyn Any>>>, sep: TokenType, open: TokenType, close: TokenType, err_action: ErrAction) -> Self where P: Parseable + 'static {
        self.functions.push(ExecFn::Nodes(
            buf,
            Box::new(|token_reader| {
                // dbg!(&token_reader.tokens[token_reader.index..]);
                match P::try_parse(token_reader) {
                    ParseResult::Ok(value) => ParseResult::Ok(Box::new(value)),
                    ParseResult::Err(err) => {
                        // dbg!(&err);
                        ParseResult::Err(err)
                    },
                    ParseResult::Skip => ParseResult::Skip
                }
            }),
            sep,
            open,
            close,
            err_action
        ));
        self
    }

    pub fn repeat_until(mut self, buf: Buf<Vec<Vec<Token>>>, divider: TokenType, until: TokenType, err_action: ErrAction) -> Self {
        self.functions.push(ExecFn::RepeatUntil(buf, divider, until, err_action));
        self
    }

    pub fn condition(mut self, condition: Box<dyn FnOnce(usize) -> ParseResult<()>>, err_action: ErrAction) -> Self {
        self.functions.push(ExecFn::Condition(condition, err_action));
        self
    }

    pub fn node<P>(mut self, buf: Buf<P>, err_action: ErrAction) -> Self where P: Parseable + Debug + 'static {
        let f = err_action.clone();
        self.functions.push(ExecFn::Node(Box::new(
            move |token_iter| {
                let x = match P::try_parse(token_iter) {
                    ParseResult::Ok(p) => {
                        dbg!(&p);
                        buf.replace(Some(p));
                        ParseResult::Ok(())
                    }
                    ParseResult::Err(err) => {
                        dbg!(&err);
                        ParseResult::Err(err)
                    }
                    ParseResult::Skip => ParseResult::Skip
                };
                dbg!(&x, f);
                x
            }
        ), err_action));
        self
    }

    pub fn cases(mut self, fns: Vec<Executor<'a>>, err_action: ErrAction) -> Self {
        self.functions.push(ExecFn::Cases(fns, err_action));
        self
    }
}

impl<'a> Executor<'a> {
    pub fn execute(mut self, token_iter: &mut TokenIter<'a>) -> ParseResult<()> {
        self.functions.reverse();
        while !self.functions.is_empty() {
            match self.functions.pop().unwrap() {
                ExecFn::Id(param, err_action) => {
                    match token_iter.id(param, err_action) {
                        ParseResult::Ok(_) => {},
                        ParseResult::Err(vec) => return ParseResult::Err(vec),
                        ParseResult::Skip => return ParseResult::Skip,
                    }
                }
                ExecFn::Divider(param, err_action) => {
                    match token_iter.divider(param, err_action) {
                        ParseResult::Ok(_) => {},
                        ParseResult::Err(vec) => return ParseResult::Err(vec),
                        ParseResult::Skip => return ParseResult::Skip,
                    }
                }
                ExecFn::Any(buf, err_action) => {
                    match token_iter.any(buf, err_action) {
                        ParseResult::Ok(_) => {},
                        ParseResult::Err(vec) => return ParseResult::Err(vec),
                        ParseResult::Skip => return ParseResult::Skip,
                    }
                }
                ExecFn::Optional(executor) => {
                    let _ = executor.execute(token_iter);
                }
                ExecFn::RepeatUntil(buf, divider, until, err_action) => {
                    match token_iter.repeat_until(buf, divider, until, err_action) {
                        ParseResult::Ok(_) => {}
                        ParseResult::Err(vec) => return ParseResult::Err(vec),
                        ParseResult::Skip => return ParseResult::Skip
                    }
                }
                ExecFn::OpenClose(buf, open, close, err_action) => {
                    match token_iter.open_close(buf, open, close, err_action) {
                        ParseResult::Ok(_) => {},
                        ParseResult::Err(vec) => return ParseResult::Err(vec),
                        ParseResult::Skip => return ParseResult::Skip,
                    }
                }
                ExecFn::Separate(buf, sep, open, close, err_action) => {
                    match token_iter.separate(buf, sep, open, close, err_action) {
                        ParseResult::Ok(_) => {},
                        ParseResult::Err(vec) => return ParseResult::Err(vec),
                        ParseResult::Skip => return ParseResult::Skip,
                    }
                }
                ExecFn::OpenCloseSeparate(buf, sep, open, close, err_action) => {
                    match token_iter.open_close_separate(buf, sep, open, close, err_action) {
                        ParseResult::Ok(_) => {},
                        ParseResult::Err(vec) => return ParseResult::Err(vec),
                        ParseResult::Skip => return ParseResult::Skip,
                    }
                }
                ExecFn::Nodes(buf, parser, sep, open, close, err_action) => {
                    match token_iter.nodes(buf, parser, sep, open, close, err_action) {
                        ParseResult::Ok(_) => {},
                        ParseResult::Err(vec) => return ParseResult::Err(vec),
                        ParseResult::Skip => return ParseResult::Skip,
                    }
                }
                ExecFn::Condition(condition, err_action) => {
                    match condition(token_iter.index) {
                        ParseResult::Ok(_) => {},
                        ParseResult::Err(vec) => return ParseResult::Err(vec),
                        ParseResult::Skip => return ParseResult::Skip,
                    }
                }
                ExecFn::Node(function, err_action) => {
                    match function(token_iter) {
                        ParseResult::Ok(_) => {dbg!("OK");},
                        ParseResult::Err(vec) => {dbg!(&vec); return ParseResult::Err(vec) },
                        ParseResult::Skip => {dbg!("SKIP"); return ParseResult::Skip },
                    }
                }
                ExecFn::Cases(executors, err_action) => {
                    let mut first: Option<ParseResult<()>> = None;
                    for executor in executors {
                        token_iter.mark_start();
                        match executor.execute(token_iter) {
                            ParseResult::Ok(_) => return ParseResult::Ok(()),
                            p if first.is_none() => first = Some(p),
                            _ => {}
                        }
                        token_iter.revert();
                    }
                    return first.expect("Must be at least 1 case");
                }
            }
        }
        ParseResult::Ok(())
    }
}

impl<'a> TokenIter<'a> {
    pub fn mark_start(&mut self) {
        self.start.push(self.index);
    }

    pub fn revert(&mut self) {
        self.index = self.start.pop().unwrap();
    }
}

impl<'a> TokenIter<'a> {
    pub fn next(&mut self) -> Option<Token> {
        if self.index >= self.tokens.len() {
            None
        } else {
            self.index += 1;
            Some(self.tokens[self.index - 1].clone())
        }
    }

    pub fn borrow_next(&mut self) -> Option<&'a Token> {
        if self.index >= self.tokens.len() {
            None
        } else {
            self.index += 1;
            Some(&self.tokens[self.index - 1])
        }
    }

    pub fn id(&mut self, param: Param<Token>, err_action: ErrAction) -> ParseResult<&mut TokenIter<'a>> {
        let result: ParseResult<_> = match self.next() {
            None => ParseResult::Err(vec![ParseError::indexed(
                "ציפה למילה, אך אין יותר אסימונים",
                self.index
            )]),
            Some(token) => match &token.token_type {
                TokenType::Identifier(id) => match param {
                    Param::Buf(buf) => {
                        buf.replace(Some(token));
                        ParseResult::Ok(self)
                    }
                    Param::Value(value) if value == id => ParseResult::Ok(self),
                    Param::Value(value) => ParseResult::Err(vec![ParseError::indexed(
                        format!("ציפה למילה '{}', אך מצא '{}'", id, value),
                        self.index
                    )])
                }
                token_type => ParseResult::Err(vec![ParseError::indexed(
                    format!("ציפה למילה, אך מצא '{}'", token_type),
                    self.index
                )])
            }
        };
        match (result, err_action) {
            (ParseResult::Ok(x), _) => ParseResult::Ok(x),
            (err, ErrAction::Error) => err,
            (_, ErrAction::Skip) => ParseResult::Skip
        }
    }

    pub fn divider(&mut self, param: Param<Token>, err_action: ErrAction) -> ParseResult<&mut TokenIter<'a>> {
        let result: ParseResult<_> = match self.next() {
            None => ParseResult::Err(vec![ParseError::indexed(
                "ציפה למחלק, אך אין יותר אסימונים",
                self.index
            )]),
            Some(token) => match &token.token_type {
                TokenType::Divider(div) => match param {
                    Param::Buf(buf) => {
                        buf.replace(Some(token));
                        ParseResult::Ok(self)
                    }
                    Param::Value(value) if value == div => ParseResult::Ok(self),
                    Param::Value(value) => {
                        self.index -= 1;
                        ParseResult::Err(vec![ParseError::indexed(
                            format!("ציפה למחלק '{}', אך מצא '{}'", div, value),
                            self.index + 1,
                        )])
                    }
                }
                token_type => {
                    self.index -= 1;
                    ParseResult::Err(vec![ParseError::indexed(
                        format!("ציפה למחלק, אך מצא '{}'", token_type),
                        self.index + 1,
                    )])
                }
            }
        };
        match (result, err_action) {
            (ParseResult::Ok(x), _) => ParseResult::Ok(x),
            (err, ErrAction::Error) => err,
            (_, ErrAction::Skip) => ParseResult::Skip
        }
    }

    pub fn any(&mut self, buf: Buf<Token>, err_action: ErrAction) -> ParseResult<&mut TokenIter<'a>> {
        let result: ParseResult<_> = match self.next() {
            None => ParseResult::Err(vec![ParseError::indexed(
                "ציפה למשהו, אך אין יותר אסימונים",
                self.index
            )]),
            Some(token) => {
                buf.replace(Some(token));
                ParseResult::Ok(self)
            }
        };
        match (result, err_action) {
            (ParseResult::Ok(x), _) => ParseResult::Ok(x),
            (err, ErrAction::Error) => err,
            (_, ErrAction::Skip) => ParseResult::Skip
        }
    }

    pub fn repeat_until(&mut self, buf: Buf<Vec<Vec<Token>>>, divider: TokenType, until: TokenType, err_action: ErrAction) -> ParseResult<&mut TokenIter<'a>> {
        let result: ParseResult<_> = {
            self.mark_start();
            let mut all_tokens: Vec<Vec<Token>> = vec![];
            let mut tokens: Vec<Token> = vec![];
            while let Some(token) = self.next() {
                if token.token_type == until {
                    all_tokens.push(tokens.drain(..).collect());
                    buf.replace(Some(all_tokens));
                    return ParseResult::Ok(self);
                } else if token.token_type == divider {
                    all_tokens.push(tokens.drain(..).collect());
                } else {
                    tokens.push(token);
                }
            }
            let err: ParseResult<_> = ParseResult::Err(vec![ParseError::indexed(
                format!("לא נמצא אסימון סופי '{until}'"),
                self.index
            )]);
            self.revert();
            err
        };
        match (result, err_action) {
            (ParseResult::Ok(x), _) => ParseResult::Ok(x),
            (err, ErrAction::Error) => err,
            (_, ErrAction::Skip) => ParseResult::Skip
        }
    }

    pub fn open_close(&mut self, buf: Buf<TokenIter<'a>>, open: TokenType, close: TokenType, err_action: ErrAction) -> ParseResult<&mut TokenIter<'a>> {
        let result: ParseResult<_> = {
            let start_index: usize = self.index;
            match self.next() {
                None => return ParseResult::Err(vec![ParseError::indexed(
                    format!("ציפה ל'{open}', אך לא נשארו אסימונים"),
                    self.index
                )]),
                Some(token) if token.token_type != open => {
                    return ParseResult::Err(vec![ParseError::indexed(
                        format!("ציפה ל'{open}', אך מצא '{}'", token.token_type),
                        self.index
                    )]);
                }
                _ => {}
            }
            let mut num_open: usize = 1;
            while let Some(token) = self.next() {
                if token.token_type == open {
                    num_open += 1;
                } else if token.token_type == close {
                    num_open -= 1;
                }

                if num_open == 0 {
                    break;
                }
            }
            if num_open > 0 {
                return ParseResult::Err(vec![ParseError::indexed(
                    format!("לא נמצא אסימון סופי '{close}'"),
                    self.index
                )]);
            }
            let token_iter: Self = Self {
                tokens: &self.tokens[..self.index - 1],
                index: start_index + 1,
                start: vec![],
            };
            // dbg!(&token_iter.tokens[token_iter.index..]);
            buf.replace(Some(token_iter));
            ParseResult::Ok(self)
        };
        match (result, err_action) {
            (ParseResult::Ok(x), _) => ParseResult::Ok(x),
            (err, ErrAction::Error) => err,
            (_, ErrAction::Skip) => ParseResult::Skip
        }
    }

    pub fn separate(&mut self, buf: Buf<Vec<TokenIter<'a>>>, sep: TokenType, open: TokenType, close: TokenType, err_action: ErrAction) -> ParseResult<&mut TokenIter<'a>> {
        let result: ParseResult<_> = {
            self.mark_start();
            let mut start_index: usize = self.index;
            let mut num_open: usize = 0;
            let mut ranges: Vec<Range<usize>> = vec![];
            while let Some(token) = self.next() {
                if token.token_type == open {
                    num_open += 1;
                } else if token.token_type == close {
                    num_open -= 1;
                }

                if num_open > 0 {
                    continue;
                }

                if token.token_type == sep {
                    ranges.push(start_index..self.index - 1);
                    start_index = self.index;
                }
            }

            ranges.push(start_index..self.index);
            // dbg!(&ranges);
            buf.replace(Some(
                ranges.into_iter()
                    .map(|range| TokenIter {
                        tokens: &self.tokens[..range.end],
                        index: range.start,
                        start: vec![]
                    })
                    .collect()
            ));
            ParseResult::Ok(self)
        };
        match (result, err_action) {
            (ParseResult::Ok(x), _) => ParseResult::Ok(x),
            (err, ErrAction::Error) => err,
            (_, ErrAction::Skip) => ParseResult::Skip
        }
    }

    pub fn open_close_separate(&mut self, buf: Buf<Vec<TokenIter<'a>>>, sep: TokenType, open: TokenType, close: TokenType, err_action: ErrAction) -> ParseResult<&mut TokenIter<'a>> {
        let open_close_buf: Buf<TokenIter<'a>> = Rc::new(RefCell::new(None));
        self.open_close(open_close_buf.clone(), open.clone(), close.clone(), err_action.clone());
        dbg!(&open_close_buf);
        open_close_buf.take().unwrap().separate(buf, sep, open, close, err_action);
        ParseResult::Ok(self)
    }

    pub fn nodes(&mut self, buf: Buf<Vec<Box<dyn Any>>>, parse: Box<dyn Fn(&mut TokenIter) -> ParseResult<Box<dyn Any>>>, sep: TokenType, open: TokenType, close: TokenType, err_action: ErrAction) -> ParseResult<&mut TokenIter<'a>> {
        let open_close_separate_buf: Buf<Vec<TokenIter<'a>>> = Rc::new(RefCell::new(None));
        self.open_close_separate(open_close_separate_buf.clone(), sep, open, close, err_action);
        dbg!(&open_close_separate_buf);
        let parse_results: Vec<_> = open_close_separate_buf.take()
            .unwrap()
            .into_iter()
            .map(|mut iter| parse(&mut iter))
            .collect();
        dbg!(&parse_results);
        for parse_result in parse_results.iter() {
            match parse_result {
                ParseResult::Err(err) => return ParseResult::Err(err.clone()),
                ParseResult::Skip => return ParseResult::Skip,
                _ => {}
            }
        }
        buf.replace(Some(
            parse_results
                .into_iter()
                .map(|result| match result {
                    ParseResult::Ok(value) => value,
                    _ => unreachable!()
                })
                .collect()
        ));
        ParseResult::Ok(self)
    }

    pub fn node<P>(&mut self, buf: Buf<P>) -> ParseResult<&mut TokenIter<'a>> where P: Parseable {
        let result = P::try_parse(self);
        match result {
            ParseResult::Ok(p) => {
                buf.replace(Some(p));
                ParseResult::Ok(self)
            }
            ParseResult::Err(errors) => ParseResult::Err(errors),
            ParseResult::Skip => ParseResult::Skip
        }
    }

    pub fn cases(&mut self, cases: Vec<Case<'a>>) -> ParseResult<&mut TokenIter<'a>> {
        let mut first_error: Option<ParseResult<&mut TokenIter>> = None;
        for case in cases.into_iter() {
            self.mark_start();
            match case(self) {
                ParseResult::Ok(value) => return ParseResult::Ok(value),
                error if first_error.is_none() => first_error = Some(error),
                _ => {}
            }
        }
        first_error.expect("Parameter 'cases' should not be empty")
    }
}

impl<'a> Debug for TokenIter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.pad("TokenIter {\n")?;
        for token in &self.tokens[self.index..] {
            f.pad(format!("\t{token:?}\n").as_str())?;
        }
        f.pad("}")
    }
}

pub type Case<'a> = Box<dyn
    FnOnce(&mut TokenIter) ->
        ParseResult<&'a mut TokenIter<'a>>
>;

pub type Buf<T> = Rc<RefCell<Option<T>>>;

pub enum Param<T> {
    Buf(Buf<T>),
    Value(&'static str)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrAction {
    Skip,
    Error
}