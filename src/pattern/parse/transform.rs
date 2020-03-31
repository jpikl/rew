use crate::pattern::parse::reader::Reader;
use crate::pattern::parse::types::ParseError;
use crate::pattern::types::{Range, Substitution, Transform};

impl Transform {
    pub fn parse(string: &str) -> Result<Self, ParseError> {
        let mut reader = Reader::new(string);

        let transform = match reader.read() {
            Some('s') => Transform::Substring(Range::parse(&mut reader)?),
            Some('S') => Transform::SubstringFromEnd(Range::parse(&mut reader)?),
            Some('r') => Transform::ReplaceFirst(Substitution::parse(&mut reader)?),
            Some('R') => Transform::ReplaceAll(Substitution::parse(&mut reader)?),
            Some('t') => Transform::Trim,
            Some('u') => Transform::LowerCase,
            Some('U') => Transform::UpperCase,
            Some('a') => Transform::ToAscii,
            Some('A') => Transform::RemoveNonAscii,
            Some('>') => Transform::LeftPad(reader.consume().to_vec()),
            Some('<') => Transform::RightPad(reader.consume().to_vec()),
            Some(_) => {
                return Err(ParseError {
                    message: "Unknown transformation",
                    position: 0,
                });
            }
            None => {
                return Err(ParseError {
                    message: "Empty input",
                    position: 0,
                })
            }
        };

        if reader.peek().is_none() {
            Ok(transform)
        } else {
            Err(ParseError {
                message: "Unexpected character",
                position: reader.posistion(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_substring() {
        assert_eq!(
            Transform::parse("s"),
            Err(ParseError {
                message: "Expected range",
                position: 1,
            })
        );
        assert_eq!(
            Transform::parse("s5"),
            Ok(Transform::Substring(Range::FromTo(5, 5)))
        );
        assert_eq!(
            Transform::parse("s2-10"),
            Ok(Transform::Substring(Range::FromTo(2, 10)))
        );
        assert_eq!(
            Transform::parse("s2-"),
            Ok(Transform::Substring(Range::From(2)))
        );
        assert_eq!(
            Transform::parse("s-10"),
            Ok(Transform::Substring(Range::To(10)))
        );
        assert_eq!(
            Transform::parse("s-"),
            Ok(Transform::Substring(Range::Full))
        );
    }

    #[test]
    fn parse_substring_from_end() {
        assert_eq!(
            Transform::parse("S"),
            Err(ParseError {
                message: "Expected range",
                position: 1,
            })
        );
        assert_eq!(
            Transform::parse("S5"),
            Ok(Transform::SubstringFromEnd(Range::FromTo(5, 5)))
        );
        assert_eq!(
            Transform::parse("S2-10"),
            Ok(Transform::SubstringFromEnd(Range::FromTo(2, 10)))
        );
        assert_eq!(
            Transform::parse("S2-"),
            Ok(Transform::SubstringFromEnd(Range::From(2)))
        );
        assert_eq!(
            Transform::parse("S-10"),
            Ok(Transform::SubstringFromEnd(Range::To(10)))
        );
        assert_eq!(
            Transform::parse("S-"),
            Ok(Transform::SubstringFromEnd(Range::Full))
        );
    }

    #[test]
    fn parse_replace_first() {
        assert_eq!(
            Transform::parse("r"),
            Err(ParseError {
                message: "Expected substitution",
                position: 1,
            })
        );
        assert_eq!(
            Transform::parse("r'ab"),
            Ok(Transform::ReplaceFirst(Substitution {
                value: "ab".to_string(),
                replacement: "".to_string()
            }))
        );
        assert_eq!(
            Transform::parse("r'ab'cd"),
            Ok(Transform::ReplaceFirst(Substitution {
                value: "ab".to_string(),
                replacement: "cd".to_string()
            }))
        );
    }

    #[test]
    fn parse_replace_all() {
        assert_eq!(
            Transform::parse("R"),
            Err(ParseError {
                message: "Expected substitution",
                position: 1,
            })
        );
        assert_eq!(
            Transform::parse("R'ab"),
            Ok(Transform::ReplaceAll(Substitution {
                value: "ab".to_string(),
                replacement: "".to_string()
            }))
        );
        assert_eq!(
            Transform::parse("R'ab'cd"),
            Ok(Transform::ReplaceAll(Substitution {
                value: "ab".to_string(),
                replacement: "cd".to_string()
            }))
        );
    }

    #[test]
    fn parse_trim() {
        assert_eq!(Transform::parse("t"), Ok(Transform::Trim));
    }

    #[test]
    fn parse_lower_case() {
        assert_eq!(Transform::parse("u"), Ok(Transform::LowerCase));
    }

    #[test]
    fn parse_upper_case() {
        assert_eq!(Transform::parse("U"), Ok(Transform::UpperCase));
    }

    #[test]
    fn parse_to_ascii() {
        assert_eq!(Transform::parse("a"), Ok(Transform::ToAscii));
    }

    #[test]
    fn parse_remove_non_ascii() {
        assert_eq!(Transform::parse("A"), Ok(Transform::RemoveNonAscii));
    }

    #[test]
    fn parse_left_pad() {
        assert_eq!(
            Transform::parse(">abc"),
            Ok(Transform::LeftPad(vec!['a', 'b', 'c']))
        );
    }

    #[test]
    fn parse_left_pad_empty() {
        assert_eq!(Transform::parse(">"), Ok(Transform::LeftPad(Vec::new())));
    }

    #[test]
    fn parse_right_pad() {
        assert_eq!(
            Transform::parse("<abc"),
            Ok(Transform::RightPad(vec!['a', 'b', 'c']))
        );
    }

    #[test]
    fn parse_right_pad_empty() {
        assert_eq!(Transform::parse("<"), Ok(Transform::RightPad(Vec::new())));
    }

    #[test]
    fn parse_unknown_transform_as_error() {
        assert_eq!(
            Transform::parse("_"),
            Err(ParseError {
                message: "Unknown transformation",
                position: 0,
            })
        );
    }

    #[test]
    fn parse_unexpected_character_as_error() {
        assert_eq!(
            Transform::parse("u_"),
            Err(ParseError {
                message: "Unexpected character",
                position: 1,
            })
        );
    }

    #[test]
    fn parse_empty_as_error() {
        assert_eq!(
            Transform::parse(""),
            Err(ParseError {
                message: "Empty input",
                position: 0,
            })
        )
    }
}
