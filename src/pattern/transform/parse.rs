use crate::pattern::char::Char;
use crate::pattern::error::ErrorType;
use crate::pattern::parse::ParseError;
use crate::pattern::range::Range;
use crate::pattern::reader::Reader;
use crate::pattern::substitution::Substitution;
use crate::pattern::transform::Transform;

impl Transform {
    pub fn parse(reader: &mut Reader) -> Result<Self, ParseError> {
        let position = reader.position();

        if let Some(char) = reader.read() {
            match char.value() {
                'n' => Ok(Transform::Substring(Range::parse(reader)?)),
                'N' => Ok(Transform::SubstringFromEnd(Range::parse(reader)?)),
                'r' => Ok(Transform::ReplaceFirst(Substitution::parse(reader)?)),
                'R' => Ok(Transform::ReplaceAll(Substitution::parse(reader)?)),
                't' => Ok(Transform::Trim),
                'l' => Ok(Transform::Lowercase),
                'u' => Ok(Transform::Uppercase),
                'a' => Ok(Transform::ToAscii),
                'A' => Ok(Transform::RemoveNonAscii),
                '<' => Ok(Transform::LeftPad(Char::join(reader.read_to_end()))),
                '>' => Ok(Transform::RightPad(Char::join(reader.read_to_end()))),
                _ => Err(ParseError {
                    typ: ErrorType::UnknownTransform(char.clone()),
                    start: position,
                    end: reader.position(),
                }),
            }
        } else {
            Err(ParseError {
                typ: ErrorType::ExpectedTransform,
                start: position,
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
            "n",
            ParseError {
                typ: ErrorType::ExpectedRange,
                start: 1,
                end: 1,
            },
        );
        assert_ok(
            "n5",
            Transform::Substring(Range {
                offset: 4,
                length: 1,
            }),
        );
        assert_ok(
            "n2-10",
            Transform::Substring(Range {
                offset: 1,
                length: 9,
            }),
        );
        assert_ok(
            "n2-",
            Transform::Substring(Range {
                offset: 1,
                length: 0,
            }),
        );
        assert_ok(
            "n-10",
            Transform::Substring(Range {
                offset: 0,
                length: 10,
            }),
        );
        assert_ok(
            "n-",
            Transform::Substring(Range {
                offset: 0,
                length: 0,
            }),
        );
    }

    #[test]
    fn substring_from_end() {
        assert_err(
            "N",
            ParseError {
                typ: ErrorType::ExpectedRange,
                start: 1,
                end: 1,
            },
        );
        assert_ok(
            "N5",
            Transform::SubstringFromEnd(Range {
                offset: 4,
                length: 1,
            }),
        );
        assert_ok(
            "N2-10",
            Transform::SubstringFromEnd(Range {
                offset: 1,
                length: 9,
            }),
        );
        assert_ok(
            "N2-",
            Transform::SubstringFromEnd(Range {
                offset: 1,
                length: 0,
            }),
        );
        assert_ok(
            "N-10",
            Transform::SubstringFromEnd(Range {
                offset: 0,
                length: 10,
            }),
        );
        assert_ok(
            "N-",
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
        assert_ok("l", Transform::Lowercase);
    }

    #[test]
    fn upper_case() {
        assert_ok("u", Transform::Uppercase);
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
        assert_ok("<abc", Transform::LeftPad("abc".to_string()));
    }

    #[test]
    fn left_pad_empty() {
        assert_ok("<", Transform::LeftPad(String::new()));
    }

    #[test]
    fn right_pad() {
        assert_ok(">abc", Transform::RightPad("abc".to_string()));
    }

    #[test]
    fn right_pad_empty() {
        assert_ok(">", Transform::RightPad(String::new()));
    }

    #[test]
    fn ignore_chars_after_transform() {
        let mut reader = Reader::from("a_");
        Transform::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 1);
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

    #[test]
    fn unknown_transform_error() {
        assert_err(
            "-_",
            ParseError {
                typ: ErrorType::UnknownTransform(Char::Raw('-')),
                start: 0,
                end: 1,
            },
        );
    }

    fn assert_ok(string: &str, transform: Transform) {
        assert_eq!(Transform::parse(&mut Reader::from(string)), Ok(transform));
    }

    fn assert_err(string: &str, error: ParseError) {
        assert_eq!(Transform::parse(&mut Reader::from(string)), Err(error));
    }
}
