use crate::pattern::char::Char;
use crate::pattern::escape::escape_str;
use crate::pattern::integer::parse_integer;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Repetition {
    pub count: usize,
    pub value: String,
}

impl Repetition {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        if reader.peek().is_none() {
            return Err(Error {
                kind: ErrorKind::ExpectedRepetition,
                range: reader.position()..reader.position(),
            });
        }

        let count = parse_integer(reader)?;
        if reader.read().is_some() {
            let value = reader.read_to_end().to_string();
            Ok(Self { count, value })
        } else {
            Err(Error {
                kind: ErrorKind::RepetitionWithoutDelimiter,
                range: reader.position()..reader.end(),
            })
        }
    }

    pub fn expand(&self) -> String {
        self.value.repeat(self.count)
    }
}

impl fmt::Display for Repetition {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}x '{}'", self.count, escape_str(&self.value))
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
        use crate::utils::ByteRange;
        use test_case::test_case;

        #[test_case("",   0..0, ErrorKind::ExpectedRepetition         ; "empty")]
        #[test_case("ab", 0..2, ErrorKind::ExpectedNumber             ; "invalid count")]
        #[test_case("12", 2..2, ErrorKind::RepetitionWithoutDelimiter ; "missing delimiter")]
        fn err(input: &str, range: ByteRange, kind: ErrorKind) {
            assert_eq!(
                Repetition::parse(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("12:",   12, ""   ; "empty value")]
        #[test_case("12:ab", 12, "ab" ; "nonempty value")]
        fn ok(input: &str, count: usize, value: &str) {
            assert_eq!(
                Repetition::parse(&mut Reader::from(input)),
                Ok(Repetition {
                    count,
                    value: value.into()
                })
            );
        }
    }

    #[test_case(0, "",   ""     ; "empty zero times")]
    #[test_case(1, "",   ""     ; "empty one time")]
    #[test_case(2, "",   ""     ; "empty multiple times")]
    #[test_case(0, "ab", ""     ; "nonempty zero times")]
    #[test_case(1, "ab", "ab"   ; "nonempty one time")]
    #[test_case(2, "ab", "abab" ; "nonempty multiple times")]
    fn expand(count: usize, value: &str, output: &str) {
        assert_eq!(
            Repetition {
                count,
                value: value.into()
            }
            .expand(),
            output
        )
    }

    #[test]
    fn display() {
        assert_eq!(
            Repetition {
                count: 5,
                value: "abc".into()
            }
            .to_string(),
            "5x 'abc'"
        );
    }
}
