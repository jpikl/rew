use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::symbols::{EXPR_END, EXPR_START, LENGTH, PIPE, RANGE};
use crate::utils::{AnyString, HasRange};
use std::convert::Infallible;
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
    ExpectedFilterOrExprEnd,
    ExpectedPipeOrExprEnd,
    ExpectedRange,
    ExpectedRangeDelimiter(Option<Char>),
    ExpectedRangeLength,
    ExpectedRegex,
    ExpectedRepetition,
    ExpectedSubstitution,
    ExpectedSwitch,
    ExprStartInsideExpr,
    IndexZero,
    IntegerOverflow(String),
    PaddingPrefixInvalid(char, Option<Char>),
    PipeOutsideExpr,
    RangeInvalid(String),
    RangeStartOverEnd(String, String),
    RegexInvalid(AnyString),
    RepetitionWithoutDelimiter,
    SubstitutionWithoutTarget(Char),
    SwitchWithoutMatcher(Char, usize),
    UnknownEscapeSequence(EscapeSequence),
    UnknownFilter(Char),
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

impl From<Infallible> for ErrorKind {
    fn from(_: Infallible) -> Self {
        unreachable!("Infallible to parse::ErrorKind conversion should never happen");
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
            Self::ExpectedFilterOrExprEnd => {
                write!(formatter, "Expected filter or closing '{}'", EXPR_END)
            }
            Self::ExpectedPipeOrExprEnd => {
                write!(formatter, "Expected '{}' or closing '{}'", PIPE, EXPR_END)
            }
            Self::ExpectedRange => write!(
                formatter,
                "Filter requires range 'A-B' or 'A+B' as a parameter"
            ),
            Self::ExpectedRangeDelimiter(None) => {
                write!(formatter, "Expected range delimiter '{}'", RANGE)
            }
            Self::ExpectedRangeDelimiter(Some(char)) => write!(
                formatter,
                "Expected range delimiter '{}' but got {}",
                RANGE, char
            ),
            Self::ExpectedRangeLength => {
                write!(formatter, "Expected range length after '{}'", LENGTH)
            }
            Self::ExpectedRegex => write!(
                formatter,
                "Filter requires regular expression as a parameter"
            ),
            Self::ExpectedRepetition => {
                write!(formatter, "Filter requires repetition 'N:V' as a parameter")
            }
            Self::ExpectedSubstitution => write!(
                formatter,
                "Filter requires substitution ':A:B' as a parameter"
            ),
            Self::ExpectedSwitch => write!(
                formatter,
                "Filter requires switch ':X1:Y1:...:Xn:Yn:D' as a parameter"
            ),
            Self::ExprStartInsideExpr => {
                write!(formatter, "Unescaped '{}' inside expression", EXPR_START)
            }
            Self::IndexZero => write!(formatter, "Indices start from 1, not 0"),
            Self::IntegerOverflow(max) => {
                write!(formatter, "Cannot parse value greater than {}", max)
            }
            Self::PaddingPrefixInvalid(fixed_prefix, None) => {
                write!(formatter, "Expected '{}' prefix or number", fixed_prefix)
            }
            Self::PaddingPrefixInvalid(expected, Some(char)) => write!(
                formatter,
                "Expected '{}' prefix or number but got {}",
                expected, char
            ),
            Self::PipeOutsideExpr => write!(formatter, "Unescaped '{}' outside expression", PIPE),
            Self::RangeInvalid(value) => write!(formatter, "Invalid range '{}'", value),
            Self::RangeStartOverEnd(start, end) => write!(
                formatter,
                "Range start {} is greater than end {}",
                start, end
            ),
            Self::RegexInvalid(value) => write!(formatter, "Invalid regular expression: {}", value),
            Self::RepetitionWithoutDelimiter => {
                write!(formatter, "Repetition is missing delimiter after number")
            }
            Self::SubstitutionWithoutTarget(char) => write!(
                formatter,
                "Substitution is missing value after delimiter {}",
                char
            ),
            Self::SwitchWithoutMatcher(char, index) => write!(
                formatter,
                "Switch is missing value after #{} delimiter {}",
                index + 1,
                char,
            ),
            Self::UnknownEscapeSequence(sequence) => write!(
                formatter,
                "Unknown escape sequence '{}{}'",
                sequence[0], sequence[1]
            ),
            Self::UnknownFilter(char) => {
                write!(formatter, "Unknown filter {}", char)
            }
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

    mod error {
        use super::*;

        #[test]
        fn range() {
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
        fn display() {
            assert_eq!(
                Error {
                    kind: ErrorKind::ExpectedNumber,
                    range: 1..2
                }
                .to_string(),
                "Invalid pattern: Expected number"
            );
        }
    }

    mod error_kind_display {
        use super::*;

        #[test]
        fn expected_filter() {
            assert_eq!(
                ErrorKind::ExpectedFilter.to_string(),
                "Expected filter after '|'"
            );
        }

        #[test]
        fn expected_number() {
            assert_eq!(ErrorKind::ExpectedNumber.to_string(), "Expected number");
        }

        #[test]
        fn expected_filter_or_expr_end() {
            assert_eq!(
                ErrorKind::ExpectedFilterOrExprEnd.to_string(),
                "Expected filter or closing '}'"
            );
        }

        #[test]
        fn expected_pipe_or_expr_end() {
            assert_eq!(
                ErrorKind::ExpectedPipeOrExprEnd.to_string(),
                "Expected '|' or closing '}'"
            );
        }

        #[test]
        fn expected_range() {
            assert_eq!(
                ErrorKind::ExpectedRange.to_string(),
                "Filter requires range 'A-B' or 'A+B' as a parameter"
            );
        }

        #[test]
        fn expected_range_delimiter() {
            assert_eq!(
                ErrorKind::ExpectedRangeDelimiter(None).to_string(),
                "Expected range delimiter '-'"
            );
            assert_eq!(
                ErrorKind::ExpectedRangeDelimiter(Some(Char::Raw('x'))).to_string(),
                "Expected range delimiter '-' but got 'x'"
            );
            assert_eq!(
                ErrorKind::ExpectedRangeDelimiter(Some(Char::Escaped('x', ['%', 'y']))).to_string(),
                "Expected range delimiter '-' but got 'x' (escape sequence '%y')"
            );
        }

        #[test]
        fn expected_range_length() {
            assert_eq!(
                ErrorKind::ExpectedRangeLength.to_string(),
                "Expected range length after '+'"
            );
        }

        #[test]
        fn expected_regex() {
            assert_eq!(
                ErrorKind::ExpectedRegex.to_string(),
                "Filter requires regular expression as a parameter"
            );
        }

        #[test]
        fn expected_repetition() {
            assert_eq!(
                ErrorKind::ExpectedRepetition.to_string(),
                "Filter requires repetition 'N:V' as a parameter"
            );
        }

        #[test]
        fn expected_substitution() {
            assert_eq!(
                ErrorKind::ExpectedSubstitution.to_string(),
                "Filter requires substitution ':A:B' as a parameter"
            );
        }

        #[test]
        fn expected_switch() {
            assert_eq!(
                ErrorKind::ExpectedSwitch.to_string(),
                "Filter requires switch ':X1:Y1:...:Xn:Yn:D' as a parameter"
            );
        }

        #[test]
        fn expr_start_inside_expr() {
            assert_eq!(
                ErrorKind::ExprStartInsideExpr.to_string(),
                "Unescaped '{' inside expression"
            );
        }

        #[test]
        fn index_zero() {
            assert_eq!(
                ErrorKind::IndexZero.to_string(),
                "Indices start from 1, not 0"
            );
        }

        #[test]
        fn integer_overflow() {
            assert_eq!(
                ErrorKind::IntegerOverflow(String::from("255")).to_string(),
                "Cannot parse value greater than 255"
            );
        }

        #[test]
        fn padding_prefix_invalid() {
            assert_eq!(
                ErrorKind::PaddingPrefixInvalid('<', None).to_string(),
                "Expected '<' prefix or number"
            );
            assert_eq!(
                ErrorKind::PaddingPrefixInvalid('<', Some(Char::Raw('x'))).to_string(),
                "Expected '<' prefix or number but got 'x'"
            );
            assert_eq!(
                ErrorKind::PaddingPrefixInvalid('<', Some(Char::Escaped('x', ['%', 'y'])))
                    .to_string(),
                "Expected '<' prefix or number but got 'x' (escape sequence '%y')"
            );
        }

        #[test]
        fn pipe_outside_expr() {
            assert_eq!(
                ErrorKind::PipeOutsideExpr.to_string(),
                "Unescaped '|' outside expression"
            );
        }

        #[test]
        fn range_invalid() {
            assert_eq!(
                ErrorKind::RangeInvalid(String::from("abc")).to_string(),
                "Invalid range 'abc'"
            );
        }

        #[test]
        fn range_start_over_end() {
            assert_eq!(
                ErrorKind::RangeStartOverEnd(String::from("2"), String::from("1")).to_string(),
                "Range start 2 is greater than end 1"
            );
        }

        #[test]
        fn regex_invalid() {
            assert_eq!(
                ErrorKind::RegexInvalid(AnyString(String::from("abc"))).to_string(),
                "Invalid regular expression: abc"
            );
        }

        #[test]
        fn repetition_without_delimiter() {
            assert_eq!(
                ErrorKind::RepetitionWithoutDelimiter.to_string(),
                "Repetition is missing delimiter after number"
            );
        }

        #[test]
        fn substitution_without_target() {
            assert_eq!(
                ErrorKind::SubstitutionWithoutTarget(Char::Raw('_')).to_string(),
                "Substitution is missing value after delimiter '_'"
            );
            assert_eq!(
                ErrorKind::SubstitutionWithoutTarget(Char::Escaped('|', ['%', '|'])).to_string(),
                "Substitution is missing value after delimiter '|' (escape sequence '%|')"
            );
        }

        #[test]
        fn swith_without_matcher() {
            assert_eq!(
                ErrorKind::SwitchWithoutMatcher(Char::Raw('_'), 0).to_string(),
                "Switch is missing value after #1 delimiter '_'"
            );
            assert_eq!(
                ErrorKind::SwitchWithoutMatcher(Char::Escaped('|', ['%', '|']), 1).to_string(),
                "Switch is missing value after #2 delimiter '|' (escape sequence '%|')"
            );
        }

        #[test]
        fn unknown_escape_sequence() {
            assert_eq!(
                ErrorKind::UnknownEscapeSequence(['%', 'x']).to_string(),
                "Unknown escape sequence '%x'"
            );
        }

        #[test]
        fn unknown_filter() {
            assert_eq!(
                ErrorKind::UnknownFilter(Char::Raw('x')).to_string(),
                "Unknown filter 'x'"
            );
            assert_eq!(
                ErrorKind::UnknownFilter(Char::Escaped('x', ['%', 'y'])).to_string(),
                "Unknown filter 'x' (escape sequence '%y')"
            );
        }

        #[test]
        fn unmatched_exprt_end() {
            assert_eq!(
                ErrorKind::UnmatchedExprEnd.to_string(),
                "No matching '{' before expression end"
            );
        }

        #[test]
        fn unmatched_exprt_start() {
            assert_eq!(
                ErrorKind::UnmatchedExprStart.to_string(),
                "No matching '}' after expression start"
            );
        }

        #[test]
        fn unterminated_escape_sequence() {
            assert_eq!(
                ErrorKind::UnterminatedEscapeSequence('%').to_string(),
                "Unterminated escape sequence '%'"
            );
        }
    }
}
