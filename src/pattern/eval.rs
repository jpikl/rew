use crate::pattern::filter::Filter;
use crate::pattern::variable::Variable;
use crate::utils::HasRange;
use std::ops::Range;
use std::path::Path;
use std::{error, fmt, result};

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
    pub range: &'a Range<usize>,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    InputNotUtf8,
}

#[derive(Debug, PartialEq)]
pub enum ErrorCause<'a> {
    Variable(&'a Variable),
    Filter(&'a Filter),
}

impl<'a> error::Error for Error<'a> {}

impl<'a> HasRange for Error<'a> {
    fn range(&self) -> &Range<usize> {
        self.range
    }
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{} evaluation failed: {}", self.cause, self.kind)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::InputNotUtf8 => write!(formatter, "Input does not have UTF-8 encoding"),
        }
    }
}

impl<'a> fmt::Display for ErrorCause<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCause::Variable(variable) => write!(formatter, "`{}` variable", variable),
            ErrorCause::Filter(filter) => write!(formatter, "`{}` filter", filter),
        }
    }
}
