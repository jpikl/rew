use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::parse::Parsed;
use crate::pattern::parser::PatternItem;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseErrorKind {
    ExpectedNumber,
    ExpectedPattern,
    ExpectedPipeOrExprEnd(Char),
    ExpectedRange,
    ExpectedSubstitution,
    ExpectedTransform,
    ExpectedVariable,
    ExprStartInsideExpr,
    RangeEndBeforeStart(usize, usize),
    RangeInvalid(String),
    RangeUnexpectedChars(String),
    RangeZeroIndex,
    RegexZeroRegexCapture,
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
            ExpectedPattern => write!(formatter, "Expected pattern but got empty string"),
            ExpectedPipeOrExprEnd(char) => {
                write!(formatter, "Expected '|' or closing '}}' but got {}", char)
            }
            ExpectedRange => write!(formatter, "Transformation requires range as a parameter"),
            ExpectedSubstitution => write!(
                formatter,
                "Transformation requires substitution as a parameter"
            ),
            ExpectedTransform => write!(formatter, "Expected transformation after '|'"),
            ExpectedVariable => write!(formatter, "Expected variable after '{{'"),
            ExprStartInsideExpr => write!(formatter, "Unescaped '{{' inside expression"),
            RangeEndBeforeStart(end, start) => write!(
                formatter,
                "Range end ({}) cannot precede its start ({})",
                end, start
            ),
            RangeInvalid(value) => write!(formatter, "Invalid range '{}'", value),
            RangeUnexpectedChars(value) => write!(
                formatter,
                "Unexpected characters '{}' in range parameter",
                value
            ),
            RangeZeroIndex => write!(formatter, "Range indice s start from 1, not 0"),
            RegexZeroRegexCapture => write!(formatter, "Regex capture groups starts from 1, not 0"),
            SubstituteWithoutValue(separator) => write!(
                formatter,
                "Substitution ({} is separator) has no value",
                separator
            ),
            UnknownEscapeSequence(seq) => {
                write!(formatter, "Unknown escape sequance '{}{}'", seq[0], seq[1])
            }
            UnknownTransform(Char::Raw(char)) => {
                write!(formatter, "Unknown transformation '{}'", char)
            }
            UnknownTransform(char) => write!(formatter, "Expected transformation but got {}", char),
            UnknownVariable(Char::Raw(char)) => write!(formatter, "Unknown variable '{}'", char),
            UnknownVariable(char) => write!(formatter, "Expected variable but got {}", char),
            UnmatchedExprEnd => write!(
                formatter,
                "End of expression'}}' does not have matching '{{'"
            ),
            UnterminatedExprStart => write!(formatter, "Unterminated start of expression '{{'"),
            UnterminatedEscapeSequence(escape) => {
                write!(formatter, "Unterminated escape sequence '{}'", escape)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct EvalError<'a> {
    pub kind: EvalErrorKind,
    pub item: &'a Parsed<PatternItem>,
}

#[derive(Debug, PartialEq)]
pub enum EvalErrorKind {
    // TODO UTF conversion error
}
