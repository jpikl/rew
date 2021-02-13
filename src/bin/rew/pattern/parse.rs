use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::symbols::{EXPR_END, EXPR_START, LENGTH, PIPE, RANGE};
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
    ExpectedFilterOrExprEnd,
    ExpectedPipeOrExprEnd,
    ExpectedRange,
    ExpectedRangeDelimiter(Option<Char>),
    ExpectedRangeLength,
    ExpectedRegex,
    ExpectedRepetition,
    ExpectedSubstitution,
    ExprStartInsideExpr,
    IntegerOverflow(String),
    PaddingPrefixInvalid(char, Option<Char>),
    PipeOutsideExpr,
    RangeIndexZero,
    RangeInvalid(String),
    RangeStartOverEnd(String, String),
    RangeLengthOverflow(String, String),
    RegexInvalid(AnyString),
    RepetitionDigitDelimiter(char),
    RepetitionWithoutDelimiter,
    SubstitutionWithoutTarget(Char),
    SubstitutionRegexInvalid(AnyString),
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
            Self::ExpectedRangeDelimiter(Some(Char::Raw(value))) => write!(
                formatter,
                "Expected range delimiter '{}' but got '{}'",
                RANGE, value
            ),
            Self::ExpectedRangeDelimiter(Some(Char::Escaped(_, sequence))) => write!(
                formatter,
                "Expected range delimiter '{}' but got escape sequence '{}{}'",
                RANGE, sequence[0], sequence[1]
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
            Self::ExprStartInsideExpr => {
                write!(formatter, "Unescaped '{}' inside expression", EXPR_START)
            }
            Self::IntegerOverflow(max) => {
                write!(formatter, "Cannot parse value greater than {}", max)
            }
            Self::PaddingPrefixInvalid(fixed_prefix, None) => {
                write!(formatter, "Expected '{}' prefix or number", fixed_prefix)
            }
            Self::PaddingPrefixInvalid(expected, Some(Char::Raw(value))) => write!(
                formatter,
                "Expected '{}' prefix or number but got '{}'",
                expected, value
            ),
            Self::PaddingPrefixInvalid(expected, Some(Char::Escaped(_, sequence))) => write!(
                formatter,
                "Expected '{}' prefix or number but got escape sequence '{}{}'",
                expected, sequence[0], sequence[1]
            ),
            Self::PipeOutsideExpr => write!(formatter, "Unescaped '{}' outside expression", PIPE),
            Self::RangeIndexZero => write!(formatter, "Range indices start from 1, not 0"),
            Self::RangeInvalid(value) => write!(formatter, "Invalid range '{}'", value),
            Self::RangeStartOverEnd(start, end) => write!(
                formatter,
                "Range start {} is greater than end {}",
                start, end
            ),
            Self::RangeLengthOverflow(length, max) => {
                write!(
                    formatter,
                    "Range length {} overflowed maximum {}",
                    length, max
                )
            }
            Self::RegexInvalid(value) => write!(formatter, "Invalid regular expression: {}", value),
            Self::RepetitionDigitDelimiter(value) => write!(
                formatter,
                "Repetition delimiter should not be a digit but is '{}'",
                value
            ),
            Self::RepetitionWithoutDelimiter => {
                write!(formatter, "Repetition is missing delimiter after number")
            }
            Self::SubstitutionWithoutTarget(Char::Raw(value)) => write!(
                formatter,
                "Substitution is missing value after delimiter '{}'",
                value
            ),
            Self::SubstitutionWithoutTarget(Char::Escaped(_, sequence)) => write!(
                formatter,
                "Substitution is missing value after delimiter '{}{}' (escape sequence)",
                sequence[0], sequence[1]
            ),
            Self::SubstitutionRegexInvalid(reason) => write!(
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
                ErrorKind::ExpectedRangeDelimiter(Some(Char::Escaped('x', ['#', 'y']))).to_string(),
                "Expected range delimiter '-' but got escape sequence '#y'"
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
        fn expr_start_inside_expr() {
            assert_eq!(
                ErrorKind::ExprStartInsideExpr.to_string(),
                "Unescaped '{' inside expression"
            );
        }

        #[test]
        fn number_overflow() {
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
                ErrorKind::PaddingPrefixInvalid('<', Some(Char::Escaped('x', ['#', 'y'])))
                    .to_string(),
                "Expected '<' prefix or number but got escape sequence '#y'"
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
        fn range_index_zero() {
            assert_eq!(
                ErrorKind::RangeIndexZero.to_string(),
                "Range indices start from 1, not 0"
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
        fn range_length_overflow() {
            assert_eq!(
                ErrorKind::RangeLengthOverflow(String::from("10"), String::from("5")).to_string(),
                "Range length 10 overflowed maximum 5"
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
        fn repetition_digit_delimiter() {
            assert_eq!(
                ErrorKind::RepetitionDigitDelimiter('0').to_string(),
                "Repetition delimiter should not be a digit but is '0'"
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
                ErrorKind::SubstitutionWithoutTarget(Char::Escaped('|', ['#', '|'])).to_string(),
                "Substitution is missing value after delimiter '#|' (escape sequence)"
            );
        }

        #[test]
        fn substitution_regex_invalid() {
            assert_eq!(
                ErrorKind::SubstitutionRegexInvalid(AnyString(String::from("abc"))).to_string(),
                "Invalid regular expression in substitution: abc"
            );
        }

        #[test]
        fn unknown_escape_sequence() {
            assert_eq!(
                ErrorKind::UnknownEscapeSequence(['#', 'x']).to_string(),
                "Unknown escape sequence '#x'"
            );
        }

        #[test]
        fn unknown_filter() {
            assert_eq!(
                ErrorKind::UnknownFilter(Char::Raw('x')).to_string(),
                "Unknown filter 'x'"
            );
            assert_eq!(
                ErrorKind::UnknownFilter(Char::Escaped('x', ['#', 'y'])).to_string(),
                "Unknown filter 'x' written as escape sequence '#y'"
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
                ErrorKind::UnterminatedEscapeSequence('#').to_string(),
                "Unterminated escape sequence '#'"
            );
        }
    }
}
