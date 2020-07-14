use crate::pattern::parse::Parsed;
use crate::pattern::parser::PatternItem;
use std::fmt;
use std::path::Path;

pub struct EvalContext<'a> {
    pub path: &'a Path,
    pub global_counter: u32,
    pub local_counter: u32,
    pub regex_captures: Option<regex::Captures<'a>>,
}

pub type EvalResult<'a, T> = Result<T, EvalError<'a>>;

#[derive(Debug, PartialEq)]
pub struct EvalError<'a> {
    pub kind: EvalErrorKind,
    pub item: &'a Parsed<PatternItem>,
}

#[derive(Debug, PartialEq)]
pub enum EvalErrorKind {
    NotUtf8,
}

impl fmt::Display for EvalErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use EvalErrorKind::*;
        match self {
            NotUtf8 => write!(formatter, "Value does not have UTF-8 encoding"),
        }
    }
}
