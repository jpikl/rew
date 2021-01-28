use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::utils::AnyString;
use regex::Regex;
use std::fmt;

#[derive(Debug)]
pub struct RegexHolder(pub Regex);

impl RegexHolder {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        let position = reader.position();
        let value = Char::join(reader.read_to_end());

        if value.is_empty() {
            return Err(Error {
                kind: ErrorKind::ExpectedRegex,
                range: position..position,
            });
        }

        match Regex::new(&value) {
            Ok(regex) => Ok(Self(regex)),
            Err(error) => Err(Error {
                kind: ErrorKind::RegexInvalid(AnyString(error.to_string())),
                range: position..reader.position(),
            }),
        }
    }

    pub fn find(&self, value: &str) -> String {
        match self.0.find(value) {
            Some(result) => result.as_str().to_string(),
            None => String::new(),
        }
    }
}

impl PartialEq for RegexHolder {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl fmt::Display for RegexHolder {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod parse {
        use super::*;

        #[test]
        fn empty() {
            let mut reader = Reader::from("");
            assert_eq!(
                RegexHolder::parse(&mut reader),
                Err(Error {
                    kind: ErrorKind::ExpectedRegex,
                    range: 0..0,
                })
            );
            assert_eq!(reader.position(), 0);
        }

        #[test]
        fn valid() {
            let mut reader = Reader::from("\\d+");
            assert_eq!(
                RegexHolder::parse(&mut reader),
                Ok(RegexHolder(Regex::new("\\d+").unwrap()))
            );
            assert_eq!(reader.position(), 3);
        }

        #[test]
        fn invalid() {
            let mut reader = Reader::from("[0-9");
            assert_eq!(
                RegexHolder::parse(&mut reader),
                Err(Error {
                    kind: ErrorKind::RegexInvalid(AnyString(String::from(
                        "This string is not compared by assertion"
                    ))),
                    range: 0..4,
                })
            );
            assert_eq!(reader.position(), 4);
        }
    }

    mod find {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(
                RegexHolder(Regex::new("\\d+").unwrap()).find(""),
                String::new()
            );
        }

        #[test]
        fn none() {
            assert_eq!(
                RegexHolder(Regex::new("\\d+").unwrap()).find("abc"),
                String::new()
            );
        }

        #[test]
        fn first() {
            assert_eq!(
                RegexHolder(Regex::new("\\d+").unwrap()).find("abc123def456"),
                String::from("123")
            );
        }
    }

    #[test]
    fn partial_eq() {
        assert_eq!(
            RegexHolder(Regex::new("[a-z]+").unwrap()),
            RegexHolder(Regex::new("[a-z]+").unwrap())
        );
        assert_ne!(
            RegexHolder(Regex::new("[a-z]+").unwrap()),
            RegexHolder(Regex::new("[a-z]*").unwrap())
        );
    }

    #[test]
    fn display() {
        assert_eq!(
            RegexHolder(Regex::new("[a-z]+").unwrap()).to_string(),
            String::from("[a-z]+")
        );
    }
}
