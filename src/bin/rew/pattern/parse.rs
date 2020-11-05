use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::symbols::{EXPR_END, EXPR_START, PIPE};
use crate::utils::{AnyString, HasRange};
use std::ops::Range;
use std::{error, fmt, result};

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
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
    SubstituteRegexInvalid(AnyString),
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
            Self::RegexCaptureZero => write!(formatter, "Regex capture groups start from 1, not 0"),
            Self::SubstituteWithoutValue(Char::Raw(value)) => write!(
                formatter,
                "Substitution is missing value after delimiter '{}'",
                value
            ),
            Self::SubstituteWithoutValue(Char::Escaped(_, sequence)) => write!(
                formatter,
                "Substitution is missing value after delimiter '{}{}' (escape sequence)",
                sequence[0], sequence[1]
            ),
            Self::SubstituteRegexInvalid(reason) => write!(
                formatter,
                "Invalid regular expression in substitution: {}",
                reason
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_range() {
        assert_eq!(
            Error {
                kind: ErrorKind::ExpectedNumber,
                range: 1..2
            }
            .range(),
            &(1..2)
        )
    }

    #[test]
    fn error_fmt() {
        assert_eq!(
            Error {
                kind: ErrorKind::ExpectedNumber,
                range: 1..2
            }
            .to_string(),
            "Invalid pattern: Expected number"
        );
    }

    #[test]
    fn error_kind_fmt() {
        assert_eq!(
            ErrorKind::ExpectedFilter.to_string(),
            "Expected filter after '|'"
        );
        assert_eq!(ErrorKind::ExpectedNumber.to_string(), "Expected number");
        assert_eq!(
            ErrorKind::ExpectedPipeOrExprEnd.to_string(),
            "Expected '|' or closing '}'"
        );
        assert_eq!(
            ErrorKind::ExpectedRange.to_string(),
            "Filter requires range as a parameter"
        );
        assert_eq!(
            ErrorKind::ExpectedSubstitution.to_string(),
            "Filter requires substitution as a parameter"
        );
        assert_eq!(
            ErrorKind::ExpectedVariable.to_string(),
            "Expected variable after '{'"
        );
        assert_eq!(
            ErrorKind::ExprStartInsideExpr.to_string(),
            "Unescaped '{' inside expression"
        );
        assert_eq!(
            ErrorKind::PipeOutsideExpr.to_string(),
            "Unescaped '|' outside expression"
        );
        assert_eq!(
            ErrorKind::RangeIndexZero.to_string(),
            "Range indices start from 1, not 0"
        );
        assert_eq!(
            ErrorKind::RangeInvalid(String::from("abc")).to_string(),
            "Invalid range 'abc'"
        );
        assert_eq!(ErrorKind::RangeUnbounded.to_string(), "Unbounded range");
        assert_eq!(
            ErrorKind::RangeStartOverEnd(2, 1).to_string(),
            "Range start (2) is bigger than end (1)"
        );
        assert_eq!(
            ErrorKind::RegexCaptureZero.to_string(),
            "Regex capture groups start from 1, not 0"
        );
        assert_eq!(
            ErrorKind::SubstituteWithoutValue(Char::Raw('_')).to_string(),
            "Substitution is missing value after delimiter '_'"
        );
        assert_eq!(
            ErrorKind::SubstituteWithoutValue(Char::Escaped('|', ['#', '|'])).to_string(),
            "Substitution is missing value after delimiter '#|' (escape sequence)"
        );
        assert_eq!(
            ErrorKind::SubstituteRegexInvalid(AnyString(String::from("abc"))).to_string(),
            "Invalid regular expression in substitution: abc"
        );
        assert_eq!(
            ErrorKind::UnknownEscapeSequence(['#', 'x']).to_string(),
            "Unknown escape sequence '#x'"
        );
        assert_eq!(
            ErrorKind::UnknownFilter(Char::Raw('x')).to_string(),
            "Unknown filter 'x'"
        );
        assert_eq!(
            ErrorKind::UnknownFilter(Char::Escaped('x', ['#', 'y'])).to_string(),
            "Unknown filter 'x' written as escape sequence '#y'"
        );
        assert_eq!(
            ErrorKind::UnknownVariable(Char::Raw('x')).to_string(),
            "Unknown variable 'x'"
        );
        assert_eq!(
            ErrorKind::UnknownVariable(Char::Escaped('x', ['#', 'y'])).to_string(),
            "Unknown variable 'x' written as escape sequence '#y'"
        );
        assert_eq!(
            ErrorKind::UnmatchedExprEnd.to_string(),
            "No matching '{' before expression end"
        );
        assert_eq!(
            ErrorKind::UnmatchedExprStart.to_string(),
            "No matching '}' after expression start"
        );
        assert_eq!(
            ErrorKind::UnterminatedEscapeSequence('#').to_string(),
            "Unterminated escape sequence '#'"
        );
    }
}
