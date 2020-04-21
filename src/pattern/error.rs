use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::lexer::{Parsed, EXPR_END, EXPR_START, PIPE};
use crate::pattern::parser::PatternItem;
use std::fmt;

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
    RangeEndBeforeStart(usize, usize),
    RangeIndexZero,
    RangeInvalid(String),
    RangeUnexpectedChars(String),
    RegexCaptureZero,
    SubstituteWithoutValue(Char),
    UnknownEscapeSequence(EscapeSequence),
    UnknownTransform(Char),
    UnknownVariable(Char),
    UnmatchedExprEnd,
    UnterminatedExprStart,
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
            RangeEndBeforeStart(end, start) => write!(
                formatter,
                "Range end ({}) cannot precede its start ({})",
                end, start
            ),
            RangeIndexZero => write!(formatter, "Range indice s start from 1, not 0"),
            RangeInvalid(value) => write!(formatter, "Invalid range '{}'", value),
            RangeUnexpectedChars(value) => write!(
                formatter,
                "Unexpected characters '{}' in range parameter",
                value
            ),
            RegexCaptureZero => write!(formatter, "Regex capture groups starts from 1, not 0"),
            SubstituteWithoutValue(Char::Raw(value)) => write!(
                formatter,
                "Substitution (where '{}' is separator) has no value",
                value
            ),
            SubstituteWithoutValue(Char::Escaped(_, sequence)) => write!(
                formatter,
                "Substitution (where escape sequence '{}{}' is separator) has no value",
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
                "End of expression'{}' does not have matching '{}'",
                EXPR_END, EXPR_START
            ),
            UnterminatedExprStart => write!(
                formatter,
                "Unterminated start of expression '{}'",
                EXPR_START
            ),
            UnterminatedEscapeSequence(escape) => {
                write!(formatter, "Unterminated escape sequence '{}'", escape)
            }
        }
    }
}

pub type EvalResult<'a, T> = Result<T, EvalError<'a>>;

#[derive(Debug, PartialEq)]
pub struct EvalError<'a> {
    pub kind: EvalErrorKind,
    pub item: &'a Parsed<PatternItem>,
}

#[derive(Debug, PartialEq)]
pub enum EvalErrorKind {
    // TODO UTF conversion error
}
