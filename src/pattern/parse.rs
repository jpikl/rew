use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::symbols::{EXPR_END, EXPR_START, PIPE};
use crate::utils::HasRange;
use std::ops::Range;
use std::{error, fmt, result};

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

impl error::Error for Error {}

impl HasRange for Error {
    fn range(&self) -> &Range<usize> {
        &self.range
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Invalid pattern: {}", self.kind)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ExpectedFilter => write!(formatter, "Expected filter after '{}'", PIPE),
            Self::ExpectedNumber => write!(formatter, "Expected number"),
            Self::ExpectedPipeOrExprEnd => {
                write!(formatter, "Expected '{}' or closing '{}'", PIPE, EXPR_END)
            }
            Self::ExpectedRange => write!(formatter, "Filter requires range as a parameter"),
            Self::ExpectedSubstitution => {
                write!(formatter, "Filter requires substitution as a parameter")
            }
            Self::ExpectedVariable => write!(formatter, "Expected variable after '{}'", EXPR_START),
            Self::ExprStartInsideExpr => {
                write!(formatter, "Unescaped '{}' inside expression", EXPR_START)
            }
            Self::PipeOutsideExpr => write!(formatter, "Unescaped '{}' outside expression", PIPE),
            Self::RangeIndexZero => write!(formatter, "Range indices start from 1, not 0"),
            Self::RangeInvalid(value) => write!(formatter, "Invalid range '{}'", value),
            Self::RangeUnbounded => write!(formatter, "Unbounded range"),
            Self::RangeStartOverEnd(start, end) => write!(
                formatter,
                "Range start ({}) is bigger than end ({})",
                start, end
            ),
            Self::RegexCaptureZero => {
                write!(formatter, "Regex capture groups starts from 1, not 0")
            }
            Self::SubstituteWithoutValue(Char::Raw(value)) => write!(
                formatter,
                "Substitution is missing value after separator '{}'",
                value
            ),
            Self::SubstituteWithoutValue(Char::Escaped(_, sequence)) => write!(
                formatter,
                "Substitution is missing value after separator '{}{}' (escape sequence)",
                sequence[0], sequence[1]
            ),
            Self::UnknownEscapeSequence(sequence) => write!(
                formatter,
                "Unknown escape sequence '{}{}'",
                sequence[0], sequence[1]
            ),
            Self::UnknownFilter(Char::Raw(value)) => {
                write!(formatter, "Unknown filter '{}'", value)
            }
            Self::UnknownFilter(Char::Escaped(value, sequence)) => write!(
                formatter,
                "Unknown filter '{}' written as escape sequence '{}{}'",
                value, sequence[0], sequence[1]
            ),
            Self::UnknownVariable(Char::Raw(char)) => {
                write!(formatter, "Unknown variable '{}'", char)
            }
            Self::UnknownVariable(Char::Escaped(value, sequence)) => write!(
                formatter,
                "Unknown variable '{}' written as escape sequence '{}{}'",
                value, sequence[0], sequence[1],
            ),
            Self::UnmatchedExprEnd => write!(
                formatter,
                "No matching '{}' before expression end",
                EXPR_START
            ),
            Self::UnmatchedExprStart => write!(
                formatter,
                "No matching '{}' after expression start",
                EXPR_END
            ),
            Self::UnterminatedEscapeSequence(escape) => {
                write!(formatter, "Unterminated escape sequence '{}'", escape)
            }
        }
    }
}
