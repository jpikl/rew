use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::escape::{escape_char, escape_str};
use crate::pattern::regex::RegexHolder;
use crate::pattern::symbols::{EXPR_END, EXPR_START, LENGTH_DELIMITER, PIPE, RANGE_DELIMITER};
use crate::utils::{AnyString, ByteRange, GetByteRange};
use std::convert::Infallible;
use std::{error, fmt, result};

pub struct Config {
    pub escape: char,
    pub separator: Separator,
}

#[cfg(test)]
impl Config {
    pub fn fixture() -> Self {
        use crate::pattern::symbols::{DEFAULT_ESCAPE, DEFAULT_SEPARATOR};
        Self {
            escape: DEFAULT_ESCAPE,
            separator: Separator::String(DEFAULT_SEPARATOR.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Separator {
    String(String),
    Regex(RegexHolder),
}

impl fmt::Display for Separator {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Separator::String(separator) => write!(formatter, "'{}'", escape_str(&separator)),
            Separator::Regex(separator) => write!(formatter, "regular expression '{}'", separator),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    pub value: T,
    pub range: ByteRange,
}

#[cfg(test)]
impl<T> From<T> for Parsed<T> {
    fn from(value: T) -> Self {
        Self { value, range: 0..0 }
    }
}

pub type Result<T> = result::Result<T, Error>;
pub type BaseResult<T> = result::Result<T, ErrorKind>;

#[derive(Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub range: ByteRange,
}

impl error::Error for Error {}

impl GetByteRange for Error {
    fn range(&self) -> &ByteRange {
        &self.range
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Invalid pattern: {}", self.kind)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorKind {
    ExpectedFieldSeparator,
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

impl From<Infallible> for ErrorKind {
    fn from(_: Infallible) -> Self {
        unreachable!("Infallible to parse::ErrorKind conversion should never happen");
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ExpectedFieldSeparator => write!(formatter, "Expected field separator"),
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
                write!(formatter, "Expected range delimiter '{}'", RANGE_DELIMITER)
            }
            Self::ExpectedRangeDelimiter(Some(char)) => write!(
                formatter,
                "Expected range delimiter '{}' but got {}",
                RANGE_DELIMITER, char
            ),
            Self::ExpectedRangeLength => {
                write!(
                    formatter,
                    "Expected range length after '{}'",
                    LENGTH_DELIMITER
                )
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
            Self::RangeInvalid(value) => {
                write!(formatter, "Invalid range '{}'", escape_str(&value))
            }
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
                escape_char(sequence[0]),
                escape_char(sequence[1])
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
                write!(
                    formatter,
                    "Unterminated escape sequence '{}'",
                    escape_char(*escape)
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(Separator::String(",".into()),   "','"                       ; "string")]
    #[test_case(Separator::Regex("\\s+".into()), "regular expression '\\s+'" ; "regex")]
    fn separator_display(separator: Separator, result: &str) {
        assert_eq!(separator.to_string(), result);
    }

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

    type E = ErrorKind;

    #[test_case(E::ExpectedFieldSeparator,                      "Expected field separator"                                   ; "expected field separator")]
    #[test_case(E::ExpectedFilter,                              "Expected filter after '|'"                                  ; "expected filter")]
    #[test_case(E::ExpectedNumber,                              "Expected number"                                            ; "expected number")]
    #[test_case(E::ExpectedFilterOrExprEnd,                     "Expected filter or closing '}'"                             ; "expected filter or expr end")]
    #[test_case(E::ExpectedPipeOrExprEnd,                       "Expected '|' or closing '}'"                                ; "expected pipe or expr end")]
    #[test_case(E::ExpectedRange,                               "Filter requires range 'A-B' or 'A+B' as a parameter"        ; "expected range")]
    #[test_case(E::ExpectedRangeDelimiter(None),                "Expected range delimiter '-'"                               ; "expected delimiter got none")]
    #[test_case(E::ExpectedRangeDelimiter(Some('x'.into())),    "Expected range delimiter '-' but got 'x'"                   ; "expected delimiter got invalid")]
    #[test_case(E::ExpectedRangeLength,                         "Expected range length after '+'"                            ; "expected range length")]
    #[test_case(E::ExpectedRegex,                               "Filter requires regular expression as a parameter"          ; "expected regex")]
    #[test_case(E::ExpectedRepetition,                          "Filter requires repetition 'N:V' as a parameter"            ; "expected repetition")]
    #[test_case(E::ExpectedSubstitution,                        "Filter requires substitution ':A:B' as a parameter"         ; "expected substitution")]
    #[test_case(E::ExpectedSwitch,                              "Filter requires switch ':X1:Y1:...:Xn:Yn:D' as a parameter" ; "expected switch")]
    #[test_case(E::ExprStartInsideExpr,                         "Unescaped '{' inside expression"                            ; "expr start inside expr")]
    #[test_case(E::IndexZero,                                   "Indices start from 1, not 0"                                ; "index zero")]
    #[test_case(E::IntegerOverflow("255".into()),               "Cannot parse value greater than 255"                        ; "integer overflow")]
    #[test_case(E::PaddingPrefixInvalid('<', None),             "Expected '<' prefix or number"                              ; "padding prefix missing")]
    #[test_case(E::PaddingPrefixInvalid('<', Some('x'.into())), "Expected '<' prefix or number but got 'x'"                  ; "padding prefix invalid")]
    #[test_case(E::PipeOutsideExpr,                             "Unescaped '|' outside expression"                           ; "pipe outside expr")]
    #[test_case(E::RangeInvalid("abc".into()),                  "Invalid range 'abc'"                                        ; "range invalid")]
    #[test_case(E::RangeStartOverEnd("2".into(), "1".into()),   "Range start 2 is greater than end 1"                        ; "range start over end")]
    #[test_case(E::RegexInvalid("abc".into()),                  "Invalid regular expression: abc"                            ; "regex invalid")]
    #[test_case(E::RepetitionWithoutDelimiter,                  "Repetition is missing delimiter after number"               ; "repetition without delimiter")]
    #[test_case(E::SubstitutionWithoutTarget('_'.into()),       "Substitution is missing value after delimiter '_'"          ; "substitution without target")]
    #[test_case(E::SwitchWithoutMatcher('_'.into(), 0),         "Switch is missing value after #1 delimiter '_'"             ; "switch without matcher")]
    #[test_case(E::UnknownEscapeSequence(['%', 'x']),           "Unknown escape sequence '%x'"                               ; "unknown escape sequence" )]
    #[test_case(E::UnknownFilter('x'.into()),                   "Unknown filter 'x'"                                         ; "unknown filter")]
    #[test_case(E::UnmatchedExprEnd,                            "No matching '{' before expression end"                      ; "unmatched expr end")]
    #[test_case(E::UnmatchedExprStart,                          "No matching '}' after expression start"                     ; "unmatched expr start")]
    #[test_case(E::UnterminatedEscapeSequence('%'),             "Unterminated escape sequence '%'"                           ; "unterminated escape sequence")]
    fn error_kind_display(kind: ErrorKind, result: &str) {
        assert_eq!(kind.to_string(), result);
    }
}
