use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::symbols::{EXPR_END, EXPR_START, PIPE};
use std::fmt;
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    pub value: T,
    pub range: Range<usize>,
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub range: Range<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseErrorKind {
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

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use ParseErrorKind::*;
        match self {
            ExpectedFilter => write!(formatter, "Expected filter after '{}'", PIPE),
            ExpectedNumber => write!(formatter, "Expected number"),
            ExpectedPipeOrExprEnd => {
                write!(formatter, "Expected '{}' or closing '{}'", PIPE, EXPR_END)
            }
            ExpectedRange => write!(formatter, "Filter requires range as a parameter"),
            ExpectedSubstitution => {
                write!(formatter, "Filter requires substitution as a parameter")
            }
            ExpectedVariable => write!(formatter, "Expected variable after '{}'", EXPR_START),
            ExprStartInsideExpr => {
                write!(formatter, "Unescaped '{}' inside expression", EXPR_START)
            }
            PipeOutsideExpr => write!(formatter, "Unescaped '{}' outside expression", PIPE),
            RangeIndexZero => write!(formatter, "Range indices start from 1, not 0"),
            RangeInvalid(value) => write!(formatter, "Invalid range '{}'", value),
            RangeUnbounded => write!(formatter, "Unbounded range"),
            RangeStartOverEnd(start, end) => write!(
                formatter,
                "Range start ({}) is bigger than end ({})",
                start, end
            ),
            RegexCaptureZero => write!(formatter, "Regex capture groups starts from 1, not 0"),
            SubstituteWithoutValue(Char::Raw(value)) => write!(
                formatter,
                "Substitution is missing value after separator '{}'",
                value
            ),
            SubstituteWithoutValue(Char::Escaped(_, sequence)) => write!(
                formatter,
                "Substitution is missing value after separator '{}{}' (escape sequence)",
                sequence[0], sequence[1]
            ),
            UnknownEscapeSequence(sequence) => write!(
                formatter,
                "Unknown escape sequence '{}{}'",
                sequence[0], sequence[1]
            ),
            UnknownFilter(Char::Raw(value)) => write!(formatter, "Unknown filter '{}'", value),
            UnknownFilter(Char::Escaped(value, sequence)) => write!(
                formatter,
                "Unknown filter '{}' written as escape sequence '{}{}'",
                value, sequence[0], sequence[1]
            ),
            UnknownVariable(Char::Raw(char)) => write!(formatter, "Unknown variable '{}'", char),
            UnknownVariable(Char::Escaped(value, sequence)) => write!(
                formatter,
                "Unknown variable '{}' written as escape sequence '{}{}'",
                value, sequence[0], sequence[1],
            ),
            UnmatchedExprEnd => write!(
                formatter,
                "No matching '{}' before expression end",
                EXPR_START
            ),
            UnmatchedExprStart => write!(
                formatter,
                "No matching '{}' after expression start",
                EXPR_END
            ),
            UnterminatedEscapeSequence(escape) => {
                write!(formatter, "Unterminated escape sequence '{}'", escape)
            }
        }
    }
}
