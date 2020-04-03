use crate::pattern::parse::ParseError;
use crate::pattern::range::Range;
use crate::pattern::reader::Reader;
use crate::pattern::substitution::Substitution;
use crate::pattern::transform::Transform;

impl Transform {
    pub fn parse(string: &str) -> Result<Self, ParseError> {
        let mut reader = Reader::new(string);
        let position = reader.position();

        let transform = match reader.read() {
            Some('s') => Transform::Substring(Range::parse(&mut reader)?),
            Some('S') => Transform::SubstringFromEnd(Range::parse(&mut reader)?),
            Some('r') => Transform::ReplaceFirst(Substitution::parse(&mut reader)?),
            Some('R') => Transform::ReplaceAll(Substitution::parse(&mut reader)?),
            Some('t') => Transform::Trim,
            Some('u') => Transform::Lowercase,
            Some('U') => Transform::Uppercase,
            Some('a') => Transform::ToAscii,
            Some('A') => Transform::RemoveNonAscii,
            Some('>') => Transform::LeftPad(reader.consume().to_vec()),
            Some('<') => Transform::RightPad(reader.consume().to_vec()),
            Some(_) => {
                return Err(ParseError {
                    message: "Unknown transformation",
                    start: position,
                    end: reader.end(),
                });
            }
            None => {
                return Err(ParseError {
                    message: "Expected transformation",
                    start: position,
                    end: reader.end(),
                })
            }
        };

        if reader.peek().is_none() {
            Ok(transform)
        } else {
            Err(ParseError {
                message: "Unexpected characters after transformation",
                start: reader.position(),
                end: reader.end(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substring() {
        assert_eq!(
            Transform::parse("s"),
            Err(ParseError {
                message: "Expected range",
                start: 1,
                end: 1,
            })
        );
        assert_eq!(
            Transform::parse("s5"),
            Ok(Transform::Substring(Range {
                offset: 4,
                length: 1
            }))
        );
        assert_eq!(
            Transform::parse("s2-10"),
            Ok(Transform::Substring(Range {
                offset: 1,
                length: 9
            }))
        );
        assert_eq!(
            Transform::parse("s2-"),
            Ok(Transform::Substring(Range {
                offset: 1,
                length: 0
            }))
        );
        assert_eq!(
            Transform::parse("s-10"),
            Ok(Transform::Substring(Range {
                offset: 0,
                length: 10
            }))
        );
        assert_eq!(
            Transform::parse("s-"),
            Ok(Transform::Substring(Range {
                offset: 0,
                length: 0
            }))
        );
    }

    #[test]
    fn substring_from_end() {
        assert_eq!(
            Transform::parse("S"),
            Err(ParseError {
                message: "Expected range",
                start: 1,
                end: 1,
            })
        );
        assert_eq!(
            Transform::parse("S5"),
            Ok(Transform::SubstringFromEnd(Range {
                offset: 4,
                length: 1
            }))
        );
        assert_eq!(
            Transform::parse("S2-10"),
            Ok(Transform::SubstringFromEnd(Range {
                offset: 1,
                length: 9
            }))
        );
        assert_eq!(
            Transform::parse("S2-"),
            Ok(Transform::SubstringFromEnd(Range {
                offset: 1,
                length: 0
            }))
        );
        assert_eq!(
            Transform::parse("S-10"),
            Ok(Transform::SubstringFromEnd(Range {
                offset: 0,
                length: 10
            }))
        );
        assert_eq!(
            Transform::parse("S-"),
            Ok(Transform::SubstringFromEnd(Range {
                offset: 0,
                length: 0
            }))
        );
    }

    #[test]
    fn replace_first() {
        assert_eq!(
            Transform::parse("r"),
            Err(ParseError {
                message: "Expected substitution",
                start: 1,
                end: 1,
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
    fn replace_all() {
        assert_eq!(
            Transform::parse("R"),
            Err(ParseError {
                message: "Expected substitution",
                start: 1,
                end: 1,
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
    fn trim() {
        assert_eq!(Transform::parse("t"), Ok(Transform::Trim));
    }

    #[test]
    fn lower_case() {
        assert_eq!(Transform::parse("u"), Ok(Transform::Lowercase));
    }

    #[test]
    fn upper_case() {
        assert_eq!(Transform::parse("U"), Ok(Transform::Uppercase));
    }

    #[test]
    fn to_ascii() {
        assert_eq!(Transform::parse("a"), Ok(Transform::ToAscii));
    }

    #[test]
    fn remove_non_ascii() {
        assert_eq!(Transform::parse("A"), Ok(Transform::RemoveNonAscii));
    }

    #[test]
    fn left_pad() {
        assert_eq!(
            Transform::parse(">abc"),
            Ok(Transform::LeftPad(vec!['a', 'b', 'c']))
        );
    }

    #[test]
    fn left_pad_empty() {
        assert_eq!(Transform::parse(">"), Ok(Transform::LeftPad(Vec::new())));
    }

    #[test]
    fn right_pad() {
        assert_eq!(
            Transform::parse("<abc"),
            Ok(Transform::RightPad(vec!['a', 'b', 'c']))
        );
    }

    #[test]
    fn right_pad_empty() {
        assert_eq!(Transform::parse("<"), Ok(Transform::RightPad(Vec::new())));
    }

    #[test]
    fn unknown_transform_error() {
        assert_eq!(
            Transform::parse("__"),
            Err(ParseError {
                message: "Unknown transformation",
                start: 0,
                end: 2,
            })
        );
    }

    #[test]
    fn unexpected_character_error() {
        assert_eq!(
            Transform::parse("u__"),
            Err(ParseError {
                message: "Unexpected characters after transformation",
                start: 1,
                end: 3,
            })
        );
    }

    #[test]
    fn empty_error() {
        assert_eq!(
            Transform::parse(""),
            Err(ParseError {
                message: "Expected transformation",
                start: 0,
                end: 0,
            })
        )
    }
}
