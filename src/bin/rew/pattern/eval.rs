use std::path::Path;
use std::{error, fmt, result};

use crate::pattern::error::{ErrorRange, GetErrorRange};
use crate::pattern::filter::Filter;
use crate::pattern::utils::AnyString;

pub type Counter = u32;

pub struct Context<'a> {
    pub working_dir: &'a Path,
    pub global_counter: Counter,
    pub local_counter: Counter,
    pub regex_captures: Option<regex::Captures<'a>>,
    pub expression_quotes: Option<char>,
}

impl<'a> Context<'a> {
    pub fn regex_capture(&self, position: usize) -> &str {
        self.regex_captures
            .as_ref()
            .and_then(|captures| captures.get(position))
            .map_or("", |capture| capture.as_str())
    }

    #[cfg(test)]
    pub fn fixture() -> Self {
        Context {
            #[cfg(unix)]
            working_dir: Path::new("/work"),
            #[cfg(windows)]
            working_dir: Path::new("C:\\work"),
            local_counter: 1,
            global_counter: 2,
            regex_captures: regex::Regex::new("(.).(.)").unwrap().captures("abc"),
            expression_quotes: None,
        }
    }
}

pub type Result<'a, T> = result::Result<T, Error<'a>>;
pub type BaseResult<T> = result::Result<T, ErrorKind>;

#[derive(Debug, PartialEq)]
pub struct Error<'a> {
    pub kind: ErrorKind,
    pub value: String,
    pub cause: &'a Filter,
    pub range: &'a ErrorRange,
}

impl<'a> error::Error for Error<'a> {}

impl<'a> GetErrorRange for Error<'a> {
    fn error_range(&self) -> &ErrorRange {
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

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    InputNotUtf8,
    CanonicalizationFailed(AnyString),
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
    use test_case::test_case;

    use super::*;

    mod eval_context_regex_capture {
        use test_case::test_case;

        use super::*;

        #[test_case(0 ; "position 0")]
        #[test_case(1 ; "position 1")]
        fn none(position: usize) {
            let mut context = Context::fixture();
            context.regex_captures = None;
            assert_eq!(context.regex_capture(position), "");
        }

        #[test_case(0, "abc" ; "position 0")]
        #[test_case(1, "a"   ; "position 1")]
        #[test_case(2, "c"   ; "position 2")]
        #[test_case(3, ""    ; "position 3")]
        fn some(number: usize, result: &str) {
            assert_eq!(Context::fixture().regex_capture(number), result);
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
                    value: "abc".into(),
                    range: &(1..2)
                }
                .error_range(),
                &(1..2)
            )
        }

        #[test]
        fn display() {
            assert_eq!(
                Error {
                    kind: ErrorKind::InputNotUtf8,
                    cause: &Filter::AbsolutePath,
                    value: "abc".into(),
                    range: &(1..2)
                }
                    .to_string(),
                "'Absolute path' evaluation failed for value 'abc': Input does not have UTF-8 encoding"
            );
        }
    }

    #[test_case(ErrorKind::InputNotUtf8,                         "Input does not have UTF-8 encoding" ; "input not utf-8")]
    #[test_case(ErrorKind::CanonicalizationFailed("abc".into()), "Path canonicalization failed: abc"  ; "canonicalization failed")]
    fn error_kind_display(kind: ErrorKind, result: &str) {
        assert_eq!(kind.to_string(), result);
    }
}
