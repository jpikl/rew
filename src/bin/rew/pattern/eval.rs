use crate::pattern::filter::Filter;
use crate::utils::{AnyString, HasRange};
use std::ops::Range;
use std::path::Path;
use std::{error, fmt, result};

pub struct Context<'a> {
    pub current_dir: &'a Path,
    pub global_counter: u32,
    pub local_counter: u32,
    pub regex_captures: Option<regex::Captures<'a>>,
}

pub type Result<'a, T> = result::Result<T, Error<'a>>;

#[derive(Debug, PartialEq)]
pub struct Error<'a> {
    pub kind: ErrorKind,
    pub value: String,
    pub cause: &'a Filter,
    pub range: &'a Range<usize>,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    InputNotUtf8,
    CanonicalizationFailed(AnyString),
}

impl<'a> error::Error for Error<'a> {}

impl<'a> HasRange for Error<'a> {
    fn range(&self) -> &Range<usize> {
        self.range
    }
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "'{}' evaluation failed for value '{}': {}",
            self.cause, self.value, self.kind
        )
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InputNotUtf8 => write!(formatter, "Input does not have UTF-8 encoding"),
            Self::CanonicalizationFailed(reason) => {
                write!(formatter, "Path canonicalization failed: {}", reason)
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
                    kind: ErrorKind::InputNotUtf8,
                    cause: &Filter::AbsolutePath,
                    value: String::from("abc"),
                    range: &(1..2)
                }
                .range(),
                &(1..2)
            )
        }

        #[test]
        fn display() {
            assert_eq!(
                Error {
                    kind: ErrorKind::InputNotUtf8,
                    cause: &Filter::AbsolutePath,
                    value: String::from("abc"),
                    range: &(1..2)
                }
                    .to_string(),
                "'Absolute path' evaluation failed for value 'abc': Input does not have UTF-8 encoding"
            );
        }
    }

    mod error_kind_display {
        use super::*;

        #[test]
        fn input_not_utf8() {
            assert_eq!(
                ErrorKind::InputNotUtf8.to_string(),
                "Input does not have UTF-8 encoding"
            );
        }

        #[test]
        fn canonicalization_failed() {
            assert_eq!(
                ErrorKind::CanonicalizationFailed(AnyString(String::from("abc"))).to_string(),
                "Path canonicalization failed: abc"
            );
        }
    }
}
