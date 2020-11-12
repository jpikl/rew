use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::utils::AnyString;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;

lazy_static! {
    static ref CAPTURE_GROUP_VAR_REGEX: Regex = Regex::new(r"\$([0-9]+)").unwrap();
}

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

    #[test]
    fn regex_holder_parse() {
        let mut reader = Reader::from("[0-9]+");
        assert_eq!(
            RegexHolder::parse(&mut reader),
            Ok(RegexHolder(Regex::new("[0-9]+").unwrap()))
        );
        assert_eq!(reader.position(), 6);
    }

    #[test]
    fn regex_holder_parse_empty_error() {
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
    fn regex_holder_parse_invalid_error() {
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

    #[test]
    fn regex_holder_eq() {
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
    fn regex_holder_fmt() {
        assert_eq!(
            RegexHolder(Regex::new("[a-z]+").unwrap()).to_string(),
            String::from("[a-z]+")
        );
    }
}
