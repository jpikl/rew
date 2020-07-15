use crate::pattern::filter::Filter;
use crate::pattern::parse::Output;
use crate::pattern::variable::Variable;
use std::fmt;
use std::path::Path;
use std::result;

pub struct Context<'a> {
    pub path: &'a Path,
    pub global_counter: u32,
    pub local_counter: u32,
    pub regex_captures: Option<regex::Captures<'a>>,
}

pub type Result<'a, T> = result::Result<T, Error<'a>>;

#[derive(Debug, PartialEq)]
pub struct Error<'a> {
    pub kind: ErrorKind,
    pub cause: ErrorCause<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    ValueNotUtf8,
}

#[derive(Debug, PartialEq)]
pub enum ErrorCause<'a> {
    Variable(&'a Output<Variable>),
    Filter(&'a Output<Filter>),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::ValueNotUtf8 => write!(formatter, "Value does not have UTF-8 encoding"),
        }
    }
}
