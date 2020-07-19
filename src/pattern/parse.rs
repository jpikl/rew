use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::symbols::{EXPR_END, EXPR_START, PIPE};
use std::fmt;
use std::ops::Range;
use std::result;

// TODO better name ... maybe Out, Tag, ...
#[derive(Debug, PartialEq)]
pub struct Output<T> {
    pub value: T,
    pub range: Range<usize>,
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorKind {
    ExpectedFilter,
    ExpectedNumber,
    ExpectedPipeOrExprEnd,
    ExpectedRange,
    ExpectedSubstitution,
    ExpectedVariable,
    ExprStartInsideExpr,
    PipeOutsideExpr,
    RangeIndexZero,
    RangeInvalid(String),
    RangeUnbounded,
    RangeStartOverEnd(usize, usize),
    RegexCaptureZero,
    SubstituteWithoutValue(Char),
    UnknownEscapeSequence(EscapeSequence),
    UnknownFilter(Char),
    UnknownVariable(Char),
    UnmatchedExprEnd,
    UnmatchedExprStart,
    UnterminatedEscapeSequence(char),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Invalid pattern: {}", self.kind)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::ExpectedFilter => write!(formatter, "Expected filter after '{}'", PIPE),
            ErrorKind::ExpectedNumber => write!(formatter, "Expected number"),
            ErrorKind::ExpectedPipeOrExprEnd => {
                write!(formatter, "Expected '{}' or closing '{}'", PIPE, EXPR_END)
            }
            ErrorKind::ExpectedRange => write!(formatter, "Filter requires range as a parameter"),
            ErrorKind::ExpectedSubstitution => {
                write!(formatter, "Filter requires substitution as a parameter")
            }
            ErrorKind::ExpectedVariable => {
                write!(formatter, "Expected variable after '{}'", EXPR_START)
            }
            ErrorKind::ExprStartInsideExpr => {
                write!(formatter, "Unescaped '{}' inside expression", EXPR_START)
            }
            ErrorKind::PipeOutsideExpr => {
                write!(formatter, "Unescaped '{}' outside expression", PIPE)
            }
            ErrorKind::RangeIndexZero => write!(formatter, "Range indices start from 1, not 0"),
            ErrorKind::RangeInvalid(value) => write!(formatter, "Invalid range '{}'", value),
            ErrorKind::RangeUnbounded => write!(formatter, "Unbounded range"),
            ErrorKind::RangeStartOverEnd(start, end) => write!(
                formatter,
                "Range start ({}) is bigger than end ({})",
                start, end
            ),
            ErrorKind::RegexCaptureZero => {
                write!(formatter, "Regex capture groups starts from 1, not 0")
            }
            ErrorKind::SubstituteWithoutValue(Char::Raw(value)) => write!(
                formatter,
                "Substitution is missing value after separator '{}'",
                value
            ),
            ErrorKind::SubstituteWithoutValue(Char::Escaped(_, sequence)) => write!(
                formatter,
                "Substitution is missing value after separator '{}{}' (escape sequence)",
                sequence[0], sequence[1]
            ),
            ErrorKind::UnknownEscapeSequence(sequence) => write!(
                formatter,
                "Unknown escape sequence '{}{}'",
                sequence[0], sequence[1]
            ),
            ErrorKind::UnknownFilter(Char::Raw(value)) => {
                write!(formatter, "Unknown filter '{}'", value)
            }
            ErrorKind::UnknownFilter(Char::Escaped(value, sequence)) => write!(
                formatter,
                "Unknown filter '{}' written as escape sequence '{}{}'",
                value, sequence[0], sequence[1]
            ),
            ErrorKind::UnknownVariable(Char::Raw(char)) => {
                write!(formatter, "Unknown variable '{}'", char)
            }
            ErrorKind::UnknownVariable(Char::Escaped(value, sequence)) => write!(
                formatter,
                "Unknown variable '{}' written as escape sequence '{}{}'",
                value, sequence[0], sequence[1],
            ),
            ErrorKind::UnmatchedExprEnd => write!(
                formatter,
                "No matching '{}' before expression end",
                EXPR_START
            ),
            ErrorKind::UnmatchedExprStart => write!(
                formatter,
                "No matching '{}' after expression start",
                EXPR_END
            ),
            ErrorKind::UnterminatedEscapeSequence(escape) => {
                write!(formatter, "Unterminated escape sequence '{}'", escape)
            }
        }
    }
}
