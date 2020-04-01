use crate::pattern::error::ParseError;
use crate::pattern::reader::Reader;

#[derive(Debug, PartialEq)]
pub struct Substitution {
    pub value: String,
    pub replacement: String,
}

impl Substitution {
    pub fn parse(reader: &mut Reader) -> Result<Self, ParseError> {
        if let Some(separator) = reader.read() {
            let mut value = String::new();
            let value_position = reader.posistion();

            while let Some(ch) = reader.read() {
                if ch == separator {
                    break;
                } else {
                    value.push(ch);
                }
            }

            if value.is_empty() {
                return Err(ParseError {
                    message: "No value to substitute",
                    position: value_position,
                });
            }

            let replacement = reader.consume().into_iter().collect();
            Ok(Substitution { value, replacement })
        } else {
            Err(ParseError {
                message: "Expected substitution",
                position: reader.posistion(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_as_error() {
        let mut reader = Reader::new("");
        assert_eq!(
            Substitution::parse(&mut reader),
            Err(ParseError {
                message: "Expected substitution",
                position: 0,
            })
        );
        assert_eq!(reader.posistion(), 0);
    }

    #[test]
    fn parse_no_value_as_error() {
        let mut reader = Reader::new("/");
        assert_eq!(
            Substitution::parse(&mut reader),
            Err(ParseError {
                message: "No value to substitute",
                position: 1,
            })
        );
        assert_eq!(reader.posistion(), 1);
    }

    #[test]
    fn parse_empty_value_as_error() {
        let mut reader = Reader::new("//");
        assert_eq!(
            Substitution::parse(&mut reader),
            Err(ParseError {
                message: "No value to substitute",
                position: 1,
            })
        );
        assert_eq!(reader.posistion(), 2);
    }

    #[test]
    fn parse_value_no_replacement() {
        let mut reader = Reader::new("/a");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: "a".to_string(),
                replacement: "".to_string()
            })
        );
        assert_eq!(reader.posistion(), 2);
    }

    #[test]
    fn parse_long_value_no_replacement() {
        let mut reader = Reader::new("/abc");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: "abc".to_string(),
                replacement: "".to_string()
            })
        );
        assert_eq!(reader.posistion(), 4);
    }

    #[test]
    fn parse_value_empty_replacement() {
        let mut reader = Reader::new("/a/");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: "a".to_string(),
                replacement: "".to_string()
            })
        );
        assert_eq!(reader.posistion(), 3);
    }

    #[test]
    fn parse_long_value_empty_replacement() {
        let mut reader = Reader::new("/abc/");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: "abc".to_string(),
                replacement: "".to_string()
            })
        );
        assert_eq!(reader.posistion(), 5);
    }

    #[test]
    fn parse_value_replacement() {
        let mut reader = Reader::new("/a/d");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: "a".to_string(),
                replacement: "d".to_string()
            })
        );
        assert_eq!(reader.posistion(), 4);
    }

    #[test]
    fn parse_long_value_replacement() {
        let mut reader = Reader::new("/abc/def");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: "abc".to_string(),
                replacement: "def".to_string()
            })
        );
        assert_eq!(reader.posistion(), 8);
    }

    #[test]
    fn parse_value_replacement_with_redundant_separators() {
        let mut reader = Reader::new("/abc/d//e/");
        assert_eq!(
            Substitution::parse(&mut reader),
            Ok(Substitution {
                value: "abc".to_string(),
                replacement: "d//e/".to_string()
            })
        );
        assert_eq!(reader.posistion(), 10);
    }
}
