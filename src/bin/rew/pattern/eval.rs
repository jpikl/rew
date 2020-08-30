use crate::pattern::filter::Filter;
use crate::pattern::variable::Variable;
use crate::utils::{AnyString, HasRange};
use std::ops::Range;
use std::path::Path;
use std::{error, fmt, result};

pub struct Context<'a> {
    pub path: &'a Path,
    pub current_dir: &'a Path,
    pub global_counter: u32,
    pub local_counter: u32,
    pub regex_captures: Option<regex::Captures<'a>>,
}

pub type Result<'a, T> = result::Result<T, Error<'a>>;

#[derive(Debug, PartialEq)]
pub struct Error<'a> {
    pub kind: ErrorKind,
    pub cause: ErrorCause<'a>,
    pub value: String,
    pub range: &'a Range<usize>,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    InputNotUtf8,
    CanonicalizationFailed(AnyString),
}

#[derive(Debug, PartialEq)]
pub enum ErrorCause<'a> {
    Variable(&'a Variable),
    Filter(&'a Filter),
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
            "{} evaluation failed for value '{}': {}",
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

impl<'a> fmt::Display for ErrorCause<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Variable(variable) => write!(formatter, "`{}` variable", variable),
            Self::Filter(filter) => write!(formatter, "`{}` filter", filter),
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
                kind: ErrorKind::InputNotUtf8,
                cause: ErrorCause::Variable(&Variable::Path),
                value: String::from("abc"),
                range: &(1..2)
            }
            .range(),
            &(1..2)
        )
    }

    #[test]
    fn error_fmt() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: ErrorKind::InputNotUtf8,
                    cause: ErrorCause::Variable(&Variable::Path),
                    value: String::from("abc"),
                    range: &(1..2)
                }
            ),
            "`Path` variable evaluation failed for value 'abc': Input does not have UTF-8 encoding"
        );
    }

    #[test]
    fn error_cause_fmt() {
        assert_eq!(
            format!("{}", ErrorCause::Variable(&Variable::Path)),
            "`Path` variable"
        );
        assert_eq!(
            format!("{}", ErrorCause::Filter(&Filter::ToLowercase)),
            "`To lowercase` filter"
        );
    }

    #[test]
    fn error_kind_fmt() {
        assert_eq!(
            format!("{}", ErrorKind::InputNotUtf8),
            "Input does not have UTF-8 encoding"
        );
        assert_eq!(
            format!(
                "{}",
                ErrorKind::CanonicalizationFailed(AnyString(String::from("abc")))
            ),
            "Path canonicalization failed: abc"
        );
    }
}
