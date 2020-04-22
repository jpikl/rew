use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::r#const::{EXPR_END, EXPR_START, PIPE};
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    pub value: T,
    pub start: usize, // TODO span: Range<usize>
    pub end: usize,
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseErrorKind {
    ExpectedNumber,
    ExpectedPipeOrExprEnd,
    ExpectedRange,
    ExpectedSubstitution,
    ExpectedTransform,
    ExpectedVariable,
    ExprStartInsideExpr,
    PipeOutsideExpr,
    RangeIndexZero,
    RangeInvalid(String),
    RangeStartOverEnd(usize, usize),
    RegexCaptureZero,
    SubstituteWithoutValue(Char),
    UnknownEscapeSequence(EscapeSequence),
    UnknownTransform(Char),
    UnknownVariable(Char),
    UnmatchedExprEnd,
    UnmatchedExprStart,
    UnterminatedEscapeSequence(char),
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use ParseErrorKind::*;
        match self {
            ExpectedNumber => write!(formatter, "Expected number"),
            ExpectedPipeOrExprEnd => {
                write!(formatter, "Expected '{}' or closing '{}'", PIPE, EXPR_END)
            }
            ExpectedRange => write!(formatter, "Transformation requires range as a parameter"),
            ExpectedSubstitution => write!(
                formatter,
                "Transformation requires substitution as a parameter"
            ),
            ExpectedTransform => write!(formatter, "Expected transformation after '{}'", PIPE),
            ExpectedVariable => write!(formatter, "Expected variable after '{}'", EXPR_START),
            ExprStartInsideExpr => {
                write!(formatter, "Unescaped '{}' inside expression", EXPR_START)
            }
            PipeOutsideExpr => write!(formatter, "Unescaped '{}' outside expression", PIPE),
            RangeIndexZero => write!(formatter, "Range indices start from 1, not 0"),
            RangeInvalid(value) => write!(formatter, "Invalid range '{}'", value),
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
                "Unknown escape sequance '{}{}'",
                sequence[0], sequence[1]
            ),
            UnknownTransform(Char::Raw(value)) => {
                write!(formatter, "Unknown transformation '{}'", value)
            }
            UnknownTransform(Char::Escaped(value, sequence)) => write!(
                formatter,
                "Unknown transformation '{}' written as escape sequence '{}{}'",
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
