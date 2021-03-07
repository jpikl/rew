use crate::pattern::filter::Filter;
use crate::utils::{AnyString, HasRange};
#[cfg(test)]
use regex::Regex;
use std::ops::Range;
use std::path::Path;
use std::{error, fmt, result};

pub struct Context<'a> {
    pub working_dir: &'a Path,
    pub global_counter: u32,
    pub local_counter: u32,
    pub regex_captures: Option<regex::Captures<'a>>,
    pub expression_quotes: Option<char>,
}

impl<'a> Context<'a> {
    pub fn regex_capture(&self, number: usize) -> &str {
        self.regex_captures
            .as_ref()
            .map(|captures| captures.get(number))
            .flatten()
            .map_or("", |capture| capture.as_str())
    }
}

pub type Result<'a, T> = result::Result<T, Error<'a>>;
pub type BaseResult<T> = result::Result<T, ErrorKind>;

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
impl<'a> Context<'a> {
    pub fn fixture() -> Self {
        Context {
            #[cfg(unix)]
            working_dir: Path::new("/work"),
            #[cfg(windows)]
            working_dir: Path::new("C:\\work"),
            local_counter: 1,
            global_counter: 2,
            regex_captures: Regex::new("(.*)").unwrap().captures("abc"),
            expression_quotes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod eval_context_regex_capture {
        use super::*;

        #[test]
        fn none() {
            let mut context = Context::fixture();
            context.regex_captures = None;
            assert_eq!(context.regex_capture(1), "");
        }

        #[test]
        fn some() {
            assert_eq!(Context::fixture().regex_capture(1), "abc");
        }

        #[test]
        fn some_invalid() {
            assert_eq!(Context::fixture().regex_capture(2), "");
        }
    }

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
