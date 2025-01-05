// use std::convert::Infallible;
// use std::ops::{ControlFlow, FromResidual, Try};

use crate::parser::prelude::*;

macro_rules! huh {
    ($x:expr) => {
        match $x {
            crate::parser::prelude::ParseResult::Ok(x) => x,
            crate::parser::prelude::ParseResult::Err(err) => return crate::parser::prelude::ParseResult::Err(err),
            crate::parser::prelude::ParseResult::Skip => return crate::parser::prelude::ParseResult::Skip
        }
    };

    ($token_iter:ident $start_index:ident $x:expr) => {
        match $x {
            crate::parser::prelude::ParseResult::Ok(x) => x,
            crate::parser::prelude::ParseResult::Err(err) =>
                result!($token_iter $start_index crate::parser::prelude::ParseResult::Err(err)),
            crate::parser::prelude::ParseResult::Skip =>
                result!($token_iter $start_index crate::parser::prelude::ParseResult::Skip)
        }
    }
}

macro_rules! skip_to_err {
    ($x:expr, $err:expr, $index:expr) => {
        match $x {
            crate::parser::prelude::ParseResult::Ok(x) => x,
            crate::parser::prelude::ParseResult::Err(err) => return crate::parser::prelude::ParseResult::Err(err),
            crate::parser::prelude::ParseResult::Skip => return crate::parser::prelude::ParseResult::Err(vec![crate::parser::prelude::ParseError::indexed($err, $index)])
        }
    };
}
pub(crate) use {huh, skip_to_err};

pub trait Parseable {
    fn try_parse(token_iter: &mut TokenIter) -> ParseResult<Self> where Self: Sized;
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseResult<T> {
    Ok(T),
    Err(Vec<ParseError>),
    Skip
}

impl<T> ParseResult<T> {
    pub fn is_ok(&self) -> bool {
        match &self {
            ParseResult::Ok(_) => true,
            _ => false
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub fn unwrap(self) -> T {
        match self {
            ParseResult::Ok(t) => t,
            _ => panic!("Unwrap failed")
        }
    }

    pub fn unwrap_err(self) -> Vec<ParseError> {
        match self {
            ParseResult::Err(err) => err,
            _ => panic!("Unwrap error failed")
        }
    }

    pub fn map<R, F>(self, f: F) -> ParseResult<R> where F: FnOnce(T) -> R {
        match self {
            ParseResult::Ok(t) => ParseResult::Ok(f(t)),
            ParseResult::Err(err) => ParseResult::Err(err),
            ParseResult::Skip => ParseResult::Skip
        }
    }
}

// impl<T, Any> FromResidual<Result<Infallible, ParseResult<Any>>> for ParseResult<T> {
//     fn from_residual(residual: Result<Infallible, ParseResult<Any>>) -> Self {
//         match residual {
//             Ok(_) => panic!("Uh oh! Spaghettios!"),
//             Err(result) => match result {
//                 ParseResult::Ok(_) => panic!("(Also) Uh oh! Spaghettios!"),
//                 ParseResult::Err(vec) => ParseResult::Err(vec),
//                 ParseResult::Skip => ParseResult::Skip
//             }
//         }
//     }
// }

// impl<T, Any> FromResidual<ParseResult<Any>> for ParseResult<T> {
//     fn from_residual(residual: ParseResult<Any>) -> Self {
//         match residual {
//             ParseResult::Ok(_) => panic!("(Also 2) Uh oh! Spaghettios!"),
//             ParseResult::Err(vec) => ParseResult::Err(vec),
//             ParseResult::Skip => ParseResult::Skip
//         }
//     }
// }

// impl<T> Try for ParseResult<T> {
//     type Output = T;
//     type Residual = ParseResult<T>;

//     fn from_output(output: Self::Output) -> Self {
//         Self::Ok(output)
//     }

//     fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
//         match self {
//             ParseResult::Ok(t) => ControlFlow::Continue(t),
//             ParseResult::Err(vec) => ControlFlow::Break(Self::Err(vec)),
//             ParseResult::Skip => ControlFlow::Break(Self::Skip)
//         }
//     }
// }