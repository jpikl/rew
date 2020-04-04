use crate::pattern::char::Char;
use crate::pattern::error::ErrorType;
use crate::pattern::parse::ParseError;
use crate::pattern::range::Range;
use crate::pattern::reader::Reader;
use crate::pattern::substitution::Substitution;
use crate::pattern::transform::Transform;

impl Transform {
    pub fn parse(chars: Vec<Char>) -> Result<Self, ParseError> {
        let mut reader = Reader::new(chars);
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
            Some('>') => Transform::LeftPad(reader.consume()),
            Some('<') => Transform::RightPad(reader.consume()),
            Some(_) => {
                return Err(ParseError {
                    typ: ErrorType::UnknownTransform,
                    start: position,
                    end: reader.position(),
                });
            }
            None => {
                return Err(ParseError {
                    typ: ErrorType::ExpectedTransform,
                    start: position,
                    end: reader.end(),
                })
            }
        };

        if reader.peek().is_none() {
            Ok(transform)
        } else {
            Err(ParseError {
                typ: ErrorType::UnexpectedCharacters,
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
        assert_err(
            "s",
            ParseError {
                typ: ErrorType::ExpectedRange,
                start: 1,
                end: 1,
            },
        );
        assert_ok(
            "s5",
            Transform::Substring(Range {
                offset: 4,
                length: 1,
            }),
        );
        assert_ok(
            "s2-10",
            Transform::Substring(Range {
                offset: 1,
                length: 9,
            }),
        );
        assert_ok(
            "s2-",
            Transform::Substring(Range {
                offset: 1,
                length: 0,
            }),
        );
        assert_ok(
            "s-10",
            Transform::Substring(Range {
                offset: 0,
                length: 10,
            }),
        );
        assert_ok(
            "s-",
            Transform::Substring(Range {
                offset: 0,
                length: 0,
            }),
        );
    }

    #[test]
    fn substring_from_end() {
        assert_err(
            "S",
            ParseError {
                typ: ErrorType::ExpectedRange,
                start: 1,
                end: 1,
            },
        );
        assert_ok(
            "S5",
            Transform::SubstringFromEnd(Range {
                offset: 4,
                length: 1,
            }),
        );
        assert_ok(
            "S2-10",
            Transform::SubstringFromEnd(Range {
                offset: 1,
                length: 9,
            }),
        );
        assert_ok(
            "S2-",
            Transform::SubstringFromEnd(Range {
                offset: 1,
                length: 0,
            }),
        );
        assert_ok(
            "S-10",
            Transform::SubstringFromEnd(Range {
                offset: 0,
                length: 10,
            }),
        );
        assert_ok(
            "S-",
            Transform::SubstringFromEnd(Range {
                offset: 0,
                length: 0,
            }),
        );
    }

    #[test]
    fn replace_first() {
        assert_err(
            "r",
            ParseError {
                typ: ErrorType::ExpectedSubstitution,
                start: 1,
                end: 1,
            },
        );
        assert_ok(
            "r'ab",
            Transform::ReplaceFirst(Substitution {
                value: "ab".to_string(),
                replacement: "".to_string(),
            }),
        );
        assert_ok(
            "r'ab'cd",
            Transform::ReplaceFirst(Substitution {
                value: "ab".to_string(),
                replacement: "cd".to_string(),
            }),
        );
    }

    #[test]
    fn replace_all() {
        assert_err(
            "R",
            ParseError {
                typ: ErrorType::ExpectedSubstitution,
                start: 1,
                end: 1,
            },
        );
        assert_ok(
            "R'ab",
            Transform::ReplaceAll(Substitution {
                value: "ab".to_string(),
                replacement: "".to_string(),
            }),
        );
        assert_ok(
            "R'ab'cd",
            Transform::ReplaceAll(Substitution {
                value: "ab".to_string(),
                replacement: "cd".to_string(),
            }),
        );
    }

    #[test]
    fn trim() {
        assert_ok("t", Transform::Trim);
    }

    #[test]
    fn lower_case() {
        assert_ok("u", Transform::Lowercase);
    }

    #[test]
    fn upper_case() {
        assert_ok("U", Transform::Uppercase);
    }

    #[test]
    fn to_ascii() {
        assert_ok("a", Transform::ToAscii);
    }

    #[test]
    fn remove_non_ascii() {
        assert_ok("A", Transform::RemoveNonAscii);
    }

    #[test]
    fn left_pad() {
        assert_ok(">abc", Transform::LeftPad("abc".to_string()));
    }

    #[test]
    fn left_pad_empty() {
        assert_ok(">", Transform::LeftPad(String::new()));
    }

    #[test]
    fn right_pad() {
        assert_ok("<abc", Transform::RightPad("abc".to_string()));
    }

    #[test]
    fn right_pad_empty() {
        assert_ok("<", Transform::RightPad(String::new()));
    }

    #[test]
    fn unknown_transform_error() {
        assert_err(
            "__",
            ParseError {
                typ: ErrorType::UnknownTransform,
                start: 0,
                end: 1,
            },
        );
    }

    #[test]
    fn unexpected_chars_error() {
        assert_err(
            "u__",
            ParseError {
                typ: ErrorType::UnexpectedCharacters,
                start: 1,
                end: 3,
            },
        );
    }

    #[test]
    fn empty_error() {
        assert_err(
            "",
            ParseError {
                typ: ErrorType::ExpectedTransform,
                start: 0,
                end: 0,
            },
        )
    }

    fn assert_ok(string: &str, transform: Transform) {
        assert_eq!(Transform::parse(Char::raw_vec(string)), Ok(transform));
    }

    fn assert_err(string: &str, error: ParseError) {
        assert_eq!(Transform::parse(Char::raw_vec(string)), Err(error));
    }
}
