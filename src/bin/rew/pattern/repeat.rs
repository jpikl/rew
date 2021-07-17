use crate::pattern::char::Char;
use crate::pattern::escape::escape_str;
use crate::pattern::integer::parse_integer;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Repetition {
    pub count: usize,
    pub value: Option<String>,
}

impl Repetition {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        Self::parse_impl(reader, false)
    }

    pub fn parse_with_delimiter(reader: &mut Reader<Char>) -> Result<Self> {
        Self::parse_impl(reader, true)
    }

    fn parse_impl(reader: &mut Reader<Char>, delimiter_required: bool) -> Result<Self> {
        if reader.peek().is_none() {
            return Err(Error {
                kind: ErrorKind::ExpectedRepetition,
                range: reader.position()..reader.position(),
            });
        }

        let count = parse_integer(reader)?;
        if reader.read().is_some() {
            let value = Some(reader.read_to_end().to_string());
            Ok(Self { count, value })
        } else if delimiter_required {
            Err(Error {
                kind: ErrorKind::RepetitionWithoutDelimiter,
                range: reader.position()..reader.end(),
            })
        } else {
            Ok(Self { count, value: None })
        }
    }

    pub fn expand(&self, default: &str) -> String {
        self.value.as_deref().unwrap_or(default).repeat(self.count)
    }
}

impl fmt::Display for Repetition {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self.value {
            None => write!(formatter, "{}x", self.count),
            Some(value) => write!(formatter, "{}x '{}'", self.count, escape_str(&value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    mod parse {
        use super::*;
        use crate::pattern::parse::{Error, ErrorKind};
        use crate::pattern::reader::Reader;
        use crate::utils::ErrorRange;
        use test_case::test_case;

        #[test_case("",   0..0, ErrorKind::ExpectedRepetition ; "empty")]
        #[test_case("ab", 0..2, ErrorKind::ExpectedNumber     ; "invalid count")]
        fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
            assert_eq!(
                Repetition::parse(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("12",    12, None       ; "missing delimiter")]
        #[test_case("12:",   12, Some("")   ; "empty value")]
        #[test_case("12:ab", 12, Some("ab") ; "nonempty value")]
        fn ok(input: &str, count: usize, value: Option<&str>) {
            assert_eq!(
                Repetition::parse(&mut Reader::from(input)),
                Ok(Repetition {
                    count,
                    value: value.map(String::from)
                })
            );
        }
    }

    mod parse_with_delimiter {
        use super::*;
        use crate::pattern::parse::{Error, ErrorKind};
        use crate::pattern::reader::Reader;
        use crate::utils::ErrorRange;
        use test_case::test_case;

        #[test_case("",   0..0, ErrorKind::ExpectedRepetition         ; "empty")]
        #[test_case("ab", 0..2, ErrorKind::ExpectedNumber             ; "invalid count")]
        #[test_case("12", 2..2, ErrorKind::RepetitionWithoutDelimiter ; "missing delimiter")]
        fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
            assert_eq!(
                Repetition::parse_with_delimiter(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("12:",   12, Some("")   ; "empty value")]
        #[test_case("12:ab", 12, Some("ab") ; "nonempty value")]
        fn ok(input: &str, count: usize, value: Option<&str>) {
            assert_eq!(
                Repetition::parse_with_delimiter(&mut Reader::from(input)),
                Ok(Repetition {
                    count,
                    value: value.map(String::from)
                })
            );
        }
    }

    #[test_case(0, None,      "",   ""      ; "default empty zero times")]
    #[test_case(1, None,      "",   ""      ; "default empty one time")]
    #[test_case(2, None,      "",   ""      ; "default empty multiple times")]
    #[test_case(0, None,      "xy", ""      ; "default nonempty zero times")]
    #[test_case(1, None,      "xy", "xy"    ; "default nonempty one time")]
    #[test_case(2, None,      "xy", "xyxy"  ; "default nonempty multiple times")]
    #[test_case(0, Some(""),  "xy",  ""     ; "value empty zero times")]
    #[test_case(1, Some(""),  "xy",  ""     ; "value empty one time")]
    #[test_case(2, Some(""),  "xy",  ""     ; "value empty multiple times")]
    #[test_case(0, Some("ab"),"xy",  ""     ; "value nonempty zero times")]
    #[test_case(1, Some("ab"),"xy",  "ab"   ; "value nonempty one time")]
    #[test_case(2, Some("ab"),"xy",  "abab" ; "value nonempty multiple times")]
    fn expand(count: usize, value: Option<&str>, default: &str, output: &str) {
        assert_eq!(
            Repetition {
                count,
                value: value.map(String::from)
            }
            .expand(default),
            output
        )
    }

    #[test_case(5, None,        "5x"       ; "repeat")]
    #[test_case(5, Some("abc"), "5x 'abc'" ; "repeat value")]
    fn display(count: usize, value: Option<&str>, result: &str) {
        assert_eq!(
            Repetition {
                count,
                value: value.map(String::from)
            }
            .to_string(),
            result
        );
    }
}
