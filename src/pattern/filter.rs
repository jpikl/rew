use crate::pattern::char::Char;
use crate::pattern::parse::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::range::Range;
use crate::pattern::reader::Reader;
use crate::pattern::substitution::Substitution;
use unidecode::unidecode;

#[derive(Debug, PartialEq)]
pub enum Filter {
    Substring(Range),
    SubstringReverse(Range),
    ReplaceFirst(Substitution),
    ReplaceAll(Substitution),
    Trim,
    Lowercase,
    Uppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(String),
    RightPad(String),
    Default(String),
}

impl Filter {
    pub fn parse(reader: &mut Reader) -> ParseResult<Self> {
        let position = reader.position();

        if let Some(char) = reader.read() {
            match char.value() {
                'n' => Ok(Filter::Substring(Range::parse(reader)?)),
                'N' => Ok(Filter::SubstringReverse(Range::parse(reader)?)),
                'r' => Ok(Filter::ReplaceFirst(Substitution::parse(reader)?)),
                'R' => Ok(Filter::ReplaceAll(Substitution::parse(reader)?)),
                't' => Ok(Filter::Trim),
                'l' => Ok(Filter::Lowercase),
                'u' => Ok(Filter::Uppercase),
                'a' => Ok(Filter::ToAscii),
                'A' => Ok(Filter::RemoveNonAscii),
                '<' => Ok(Filter::LeftPad(Char::join(reader.read_to_end()))),
                '>' => Ok(Filter::RightPad(Char::join(reader.read_to_end()))),
                'd' => Ok(Filter::Default(Char::join(reader.read_to_end()))),
                _ => Err(ParseError {
                    kind: ParseErrorKind::UnknownFilter(char.clone()),
                    start: position,
                    end: reader.position(),
                }),
            }
        } else {
            Err(ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                start: position,
                end: reader.end(),
            })
        }
    }

    pub fn apply(&self, mut string: String) -> String {
        match self {
            Filter::Substring(range) => {
                if let Some(start) = range.start() {
                    if let Some((start, _)) = string.char_indices().nth(start) {
                        string.replace_range(..start, "");
                    } else {
                        string.clear();
                    }
                }
                if let Some(length) = range.length() {
                    if let Some((end, _)) = string.char_indices().nth(length) {
                        string.replace_range(end.., "");
                    }
                }
                string
            }

            Filter::SubstringReverse(range) => {
                if let Some(start) = range.start() {
                    if start > 0 {
                        if let Some((start, _)) = string.char_indices().nth_back(start - 1) {
                            string.replace_range(start.., "");
                        } else {
                            string.clear();
                        }
                    }
                }
                if let Some(length) = range.length() {
                    if length > 0 {
                        if let Some((end, _)) = string.char_indices().nth_back(length - 1) {
                            string.replace_range(..end, "");
                        }
                    } else {
                        string.clear();
                    }
                }
                string
            }

            Filter::ReplaceFirst(Substitution { value, replacement }) => {
                string.replacen(value, replacement, 1)
            }

            Filter::ReplaceAll(Substitution { value, replacement }) => {
                string.replace(value, replacement)
            }

            Filter::Trim => string.trim().to_string(),
            Filter::Lowercase => string.to_lowercase(),
            Filter::Uppercase => string.to_uppercase(),
            Filter::ToAscii => unidecode(&string),

            Filter::RemoveNonAscii => {
                string.retain(|ch| ch.is_ascii());
                string
            }

            Filter::LeftPad(padding) => {
                for char in padding.chars().rev().skip(string.len()) {
                    string.insert(0, char);
                }
                string
            }

            Filter::RightPad(padding) => {
                for char in padding.chars().skip(string.len()) {
                    string.push(char);
                }
                string
            }

            Filter::Default(default) => {
                if string.is_empty() {
                    string.push_str(default);
                }
                string
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;

    #[test]
    fn parse_substring() {
        assert_err(
            "n",
            ParseError {
                kind: ParseErrorKind::ExpectedRange,
                start: 1,
                end: 1,
            },
        );
        assert_ok("n5", Filter::Substring(Range::FromTo(4, 5)));
        assert_ok("n2-10", Filter::Substring(Range::FromTo(1, 10)));
        assert_ok("n2-", Filter::Substring(Range::From(1)));
        assert_ok("n-10", Filter::Substring(Range::To(10)));
        assert_ok("n-", Filter::Substring(Range::Full));
    }

    #[test]
    fn parse_substring_from_end() {
        assert_err(
            "N",
            ParseError {
                kind: ParseErrorKind::ExpectedRange,
                start: 1,
                end: 1,
            },
        );
        assert_ok("N5", Filter::SubstringReverse(Range::FromTo(4, 5)));
        assert_ok("N2-10", Filter::SubstringReverse(Range::FromTo(1, 10)));
        assert_ok("N2-", Filter::SubstringReverse(Range::From(1)));
        assert_ok("N-10", Filter::SubstringReverse(Range::To(10)));
        assert_ok("N-", Filter::SubstringReverse(Range::Full));
    }

    #[test]
    fn parse_replace_first() {
        assert_err(
            "r",
            ParseError {
                kind: ParseErrorKind::ExpectedSubstitution,
                start: 1,
                end: 1,
            },
        );
        assert_ok(
            "r'ab",
            Filter::ReplaceFirst(Substitution {
                value: "ab".to_string(),
                replacement: "".to_string(),
            }),
        );
        assert_ok(
            "r'ab'cd",
            Filter::ReplaceFirst(Substitution {
                value: "ab".to_string(),
                replacement: "cd".to_string(),
            }),
        );
    }

    #[test]
    fn parse_replace_all() {
        assert_err(
            "R",
            ParseError {
                kind: ParseErrorKind::ExpectedSubstitution,
                start: 1,
                end: 1,
            },
        );
        assert_ok(
            "R'ab",
            Filter::ReplaceAll(Substitution {
                value: "ab".to_string(),
                replacement: "".to_string(),
            }),
        );
        assert_ok(
            "R'ab'cd",
            Filter::ReplaceAll(Substitution {
                value: "ab".to_string(),
                replacement: "cd".to_string(),
            }),
        );
    }

    #[test]
    fn parse_trim() {
        assert_ok("t", Filter::Trim);
    }

    #[test]
    fn parse_lower_case() {
        assert_ok("l", Filter::Lowercase);
    }

    #[test]
    fn parse_upper_case() {
        assert_ok("u", Filter::Uppercase);
    }

    #[test]
    fn parse_to_ascii() {
        assert_ok("a", Filter::ToAscii);
    }

    #[test]
    fn parse_remove_non_ascii() {
        assert_ok("A", Filter::RemoveNonAscii);
    }

    #[test]
    fn parse_left_pad() {
        assert_ok("<abc", Filter::LeftPad("abc".to_string()));
    }

    #[test]
    fn parse_left_pad_empty() {
        assert_ok("<", Filter::LeftPad(String::new()));
    }

    #[test]
    fn parse_right_pad() {
        assert_ok(">abc", Filter::RightPad("abc".to_string()));
    }

    #[test]
    fn parse_right_pad_empty() {
        assert_ok(">", Filter::RightPad(String::new()));
    }

    #[test]
    fn parse_default() {
        assert_ok("dabc", Filter::Default("abc".to_string()));
    }

    #[test]
    fn parse_default_empty() {
        assert_ok("d", Filter::Default(String::new()));
    }

    #[test]
    fn parse_ignore_chars_after_filter() {
        let mut reader = Reader::from("a_");
        Filter::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_empty_error() {
        assert_err(
            "",
            ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                start: 0,
                end: 0,
            },
        )
    }

    #[test]
    fn parse_unknown_filter_error() {
        assert_err(
            "-_",
            ParseError {
                kind: ParseErrorKind::UnknownFilter(Char::Raw('-')),
                start: 0,
                end: 1,
            },
        );
    }

    // TODO replace by inline assert_eq!
    fn assert_ok(string: &str, filter: Filter) {
        assert_eq!(Filter::parse(&mut Reader::from(string)), Ok(filter));
    }

    // TODO replace by inline assert_eq!
    fn assert_err(string: &str, error: ParseError) {
        assert_eq!(Filter::parse(&mut Reader::from(string)), Err(error));
    }

    #[test]
    fn apply_substring_full() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::Full).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_from_first() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::From(0)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_from_last() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::From(3)).apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn apply_substring_from_over() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::From(4)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_to_below() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::To(0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_to_last_but_one() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::To(3)).apply(string);
        assert_eq!(string, "ábč");
    }

    #[test]
    fn apply_substring_to_last() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::To(4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_to_over() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::To(5)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_from_first_to_below() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::FromTo(0, 0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_from_first_to_last_but_one() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::FromTo(0, 3)).apply(string);
        assert_eq!(string, "ábč");
    }

    #[test]
    fn apply_substring_from_first_to_last() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::FromTo(0, 4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_from_last_to_last() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::FromTo(3, 4)).apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn apply_substring_from_last_to_over() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::FromTo(3, 5)).apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn apply_substring_from_over_to_over() {
        let mut string = "ábčd".to_string();
        string = Filter::Substring(Range::FromTo(4, 5)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_reverse_full() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::Full).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_from_first() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::From(0)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_from_last() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::From(3)).apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn apply_substring_reverse_from_over() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::From(4)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_reverse_to_below() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::To(0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_reverse_to_last_but_one() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::To(3)).apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn apply_substring_reverse_to_last() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::To(4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_to_over() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::To(5)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_from_first_to_below() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::FromTo(0, 0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_reverse_from_first_to_last_but_one() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::FromTo(0, 3)).apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn apply_substring_reverse_from_first_to_last() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::FromTo(0, 4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_from_last_to_last() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::FromTo(3, 4)).apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn apply_substring_reverse_from_last_to_over() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::FromTo(3, 5)).apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn apply_substring_reverse_from_over_to_over() {
        let mut string = "ábčd".to_string();
        string = Filter::SubstringReverse(Range::FromTo(4, 5)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_replace_first() {
        let mut string = "abcd_abcd".to_string();
        string = Filter::ReplaceFirst(Substitution {
            value: "ab".to_string(),
            replacement: "x".to_string(),
        })
        .apply(string);
        assert_eq!(string, "xcd_abcd");
    }

    #[test]
    fn apply_replace_all() {
        let mut string = "abcd_abcd".to_string();
        string = Filter::ReplaceAll(Substitution {
            value: "ab".to_string(),
            replacement: "x".to_string(),
        })
        .apply(string);
        assert_eq!(string, "xcd_xcd");
    }

    #[test]
    fn apply_remove_first() {
        let mut string = "abcd_abcd".to_string();
        string = Filter::ReplaceFirst(Substitution {
            value: "ab".to_string(),
            replacement: String::new(),
        })
        .apply(string);
        assert_eq!(string, "cd_abcd");
    }

    #[test]
    fn apply_remove_all() {
        let mut string = "abcd_abcd".to_string();
        string = Filter::ReplaceAll(Substitution {
            value: "ab".to_string(),
            replacement: String::new(),
        })
        .apply(string);
        assert_eq!(string, "cd_cd");
    }

    #[test]
    fn apply_trim_none() {
        let mut string = "abcd".to_string();
        string = Filter::Trim.apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn apply_trim() {
        let mut string = " abcd ".to_string();
        string = Filter::Trim.apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn apply_lowercase() {
        let mut string = "ábčdÁBČD".to_string();
        string = Filter::Lowercase.apply(string);
        assert_eq!(string, "ábčdábčd");
    }

    #[test]
    fn apply_uppercase() {
        let mut string = "ábčdÁBČD".to_string();
        string = Filter::Uppercase.apply(string);
        assert_eq!(string, "ÁBČDÁBČD");
    }

    #[test]
    fn apply_to_ascii() {
        let mut string = "ábčdÁBČD".to_string();
        string = Filter::ToAscii.apply(string);
        assert_eq!(string, "abcdABCD");
    }

    #[test]
    fn apply_remove_non_ascii() {
        let mut string = "ábčdÁBČD".to_string();
        string = Filter::RemoveNonAscii.apply(string);
        assert_eq!(string, "bdBD");
    }

    #[test]
    fn apply_left_pad_all() {
        let mut string = "".to_string();
        string = Filter::LeftPad("0123".to_string()).apply(string);
        assert_eq!(string, "0123");
    }

    #[test]
    fn apply_left_pad_some() {
        let mut string = "ab".to_string();
        string = Filter::LeftPad("0123".to_string()).apply(string);
        assert_eq!(string, "01ab");
    }

    #[test]
    fn apply_left_pad_none() {
        let mut string = "abcd".to_string();
        string = Filter::LeftPad("0123".to_string()).apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn apply_right_pad_all() {
        let mut string = "".to_string();
        string = Filter::RightPad("0123".to_string()).apply(string);
        assert_eq!(string, "0123");
    }

    #[test]
    fn apply_right_pad_some() {
        let mut string = "ab".to_string();
        string = Filter::RightPad("0123".to_string()).apply(string);
        assert_eq!(string, "ab23");
    }

    #[test]
    fn apply_right_pad_none() {
        let mut string = "abcd".to_string();
        string = Filter::RightPad("0123".to_string()).apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn apply_default_used() {
        let mut string = "".to_string();
        string = Filter::Default("xyz".to_string()).apply(string);
        assert_eq!(string, "xyz");
    }

    #[test]
    fn apply_default_unused() {
        let mut string = "abcd".to_string();
        string = Filter::Default("xyz".to_string()).apply(string);
        assert_eq!(string, "abcd");
    }
}
