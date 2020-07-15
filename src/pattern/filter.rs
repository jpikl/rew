use crate::pattern::char::{AsChar, Char};
use crate::pattern::parse::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::range::Range;
use crate::pattern::reader::Reader;
use crate::pattern::substitution::Substitution;
use std::fmt;
use unidecode::unidecode;

#[derive(Debug, PartialEq)]
pub enum Filter {
    Substring(Range),
    SubstringReverse(Range),
    ReplaceFirst(Substitution),
    ReplaceAll(Substitution),
    Trim,
    ToLowercase,
    ToUppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(String),
    RightPad(String),
    Default(String),
}

impl Filter {
    pub fn parse(reader: &mut Reader<Char>) -> ParseResult<Self> {
        let position = reader.position();

        if let Some(char) = reader.read() {
            match char.as_char() {
                'n' => Ok(Filter::Substring(Range::parse(reader)?)),
                'N' => Ok(Filter::SubstringReverse(Range::parse(reader)?)),
                'r' => Ok(Filter::ReplaceFirst(Substitution::parse(reader)?)),
                'R' => Ok(Filter::ReplaceAll(Substitution::parse(reader)?)),
                // TODO 's' RegexReplaceFirst
                // TODO 'S' RegexReplaceAll
                't' => Ok(Filter::Trim),
                'l' => Ok(Filter::ToLowercase),
                'u' => Ok(Filter::ToUppercase),
                'a' => Ok(Filter::ToAscii),
                'A' => Ok(Filter::RemoveNonAscii),
                '<' => Ok(Filter::LeftPad(Char::join(reader.read_to_end()))),
                '>' => Ok(Filter::RightPad(Char::join(reader.read_to_end()))),
                'd' => Ok(Filter::Default(Char::join(reader.read_to_end()))),
                // TODO 'e' ExternalCommand
                _ => Err(ParseError {
                    kind: ParseErrorKind::UnknownFilter(char.clone()),
                    range: position..reader.position(),
                }),
            }
        } else {
            Err(ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                range: position..reader.end(),
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
            Filter::ToLowercase => string.to_lowercase(),
            Filter::ToUppercase => string.to_uppercase(),
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

impl fmt::Display for Filter {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Filter::Substring(range) => write!(formatter, "Substring {}", range),
            Filter::SubstringReverse(range) => {
                write!(formatter, "Substring (reverse indexing) {}", range)
            }
            Filter::ReplaceFirst(substitution) => {
                write!(formatter, "Replace first {}", substitution)
            }
            Filter::ReplaceAll(substitution) => write!(formatter, "Replace all {}", substitution),
            Filter::Default(replacement) => write!(formatter, "Replace empty by '{}'", replacement),
            Filter::Trim => write!(formatter, "Trim"),
            Filter::ToLowercase => write!(formatter, "To lowercase"),
            Filter::ToUppercase => write!(formatter, "To uppercase"),
            Filter::ToAscii => write!(formatter, "To ASCII"),
            Filter::RemoveNonAscii => write!(formatter, "Remove non-ASCII"),
            Filter::LeftPad(padding) => write!(formatter, "Left pad with '{}'", padding),
            Filter::RightPad(padding) => write!(formatter, "Right pad with '{}'", padding),
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
        assert_eq!(
            parse("n"),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedRange,
                range: 1..1,
            }),
        );
        assert_eq!(parse("n5"), Ok(Filter::Substring(Range::FromTo(4, 5))));
        assert_eq!(parse("n2-10"), Ok(Filter::Substring(Range::FromTo(1, 10))));
        assert_eq!(parse("n2-"), Ok(Filter::Substring(Range::From(1))));
        assert_eq!(parse("n-10"), Ok(Filter::Substring(Range::To(10))));
        assert_eq!(parse("n-"), Ok(Filter::Substring(Range::Full)));
    }

    #[test]
    fn parse_substring_from_end() {
        assert_eq!(
            parse("N"),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedRange,
                range: 1..1,
            }),
        );
        assert_eq!(
            parse("N5"),
            Ok(Filter::SubstringReverse(Range::FromTo(4, 5)))
        );
        assert_eq!(
            parse("N2-10"),
            Ok(Filter::SubstringReverse(Range::FromTo(1, 10)))
        );
        assert_eq!(parse("N2-"), Ok(Filter::SubstringReverse(Range::From(1))));
        assert_eq!(parse("N-10"), Ok(Filter::SubstringReverse(Range::To(10))));
        assert_eq!(parse("N-"), Ok(Filter::SubstringReverse(Range::Full)));
    }

    #[test]
    fn parse_replace_first() {
        assert_eq!(
            parse("r"),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedSubstitution,
                range: 1..1,
            }),
        );
        assert_eq!(
            parse("r_ab"),
            Ok(Filter::ReplaceFirst(Substitution {
                value: String::from("ab"),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("r_ab_cd"),
            Ok(Filter::ReplaceFirst(Substitution {
                value: String::from("ab"),
                replacement: String::from("cd"),
            })),
        );
    }

    #[test]
    fn parse_replace_all() {
        assert_eq!(
            parse("R"),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedSubstitution,
                range: 1..1,
            }),
        );
        assert_eq!(
            parse("R_ab"),
            Ok(Filter::ReplaceAll(Substitution {
                value: String::from("ab"),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("R_ab_cd"),
            Ok(Filter::ReplaceAll(Substitution {
                value: String::from("ab"),
                replacement: String::from("cd"),
            })),
        );
    }

    #[test]
    fn parse_trim() {
        assert_eq!(parse("t"), Ok(Filter::Trim));
    }

    #[test]
    fn parse_to_lower_case() {
        assert_eq!(parse("l"), Ok(Filter::ToLowercase));
    }

    #[test]
    fn parse_to_upper_case() {
        assert_eq!(parse("u"), Ok(Filter::ToUppercase));
    }

    #[test]
    fn parse_to_ascii() {
        assert_eq!(parse("a"), Ok(Filter::ToAscii));
    }

    #[test]
    fn parse_remove_non_ascii() {
        assert_eq!(parse("A"), Ok(Filter::RemoveNonAscii));
    }

    #[test]
    fn parse_left_pad() {
        assert_eq!(parse("<abc"), Ok(Filter::LeftPad(String::from("abc"))));
    }

    #[test]
    fn parse_left_pad_empty() {
        assert_eq!(parse("<"), Ok(Filter::LeftPad(String::new())));
    }

    #[test]
    fn parse_right_pad() {
        assert_eq!(parse(">abc"), Ok(Filter::RightPad(String::from("abc"))));
    }

    #[test]
    fn parse_right_pad_empty() {
        assert_eq!(parse(">"), Ok(Filter::RightPad(String::new())));
    }

    #[test]
    fn parse_default() {
        assert_eq!(parse("dabc"), Ok(Filter::Default(String::from("abc"))));
    }

    #[test]
    fn parse_default_empty() {
        assert_eq!(parse("d"), Ok(Filter::Default(String::new())));
    }

    #[test]
    fn parse_ignore_chars_after_filter() {
        let mut reader = Reader::from("a_");
        Filter::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_empty_error() {
        assert_eq!(
            parse(""),
            Err(ParseError {
                kind: ParseErrorKind::ExpectedFilter,
                range: 0..0,
            }),
        )
    }

    #[test]
    fn parse_unknown_filter_error() {
        assert_eq!(
            parse("-_"),
            Err(ParseError {
                kind: ParseErrorKind::UnknownFilter(Char::Raw('-')),
                range: 0..1,
            }),
        );
    }

    fn parse(string: &str) -> ParseResult<Filter> {
        Filter::parse(&mut Reader::from(string))
    }

    #[test]
    fn apply_substring_full() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::Full).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_from_first() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::From(0)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_from_last() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::From(3)).apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn apply_substring_from_over() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::From(4)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_to_below() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::To(0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_to_last_but_one() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::To(3)).apply(string);
        assert_eq!(string, "ábč");
    }

    #[test]
    fn apply_substring_to_last() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::To(4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_to_over() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::To(5)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_from_first_to_below() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::FromTo(0, 0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_from_first_to_last_but_one() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::FromTo(0, 3)).apply(string);
        assert_eq!(string, "ábč");
    }

    #[test]
    fn apply_substring_from_first_to_last() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::FromTo(0, 4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_from_last_to_last() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::FromTo(3, 4)).apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn apply_substring_from_last_to_over() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::FromTo(3, 5)).apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn apply_substring_from_over_to_over() {
        let mut string = String::from("ábčd");
        string = Filter::Substring(Range::FromTo(4, 5)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_reverse_full() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::Full).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_from_first() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::From(0)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_from_last() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::From(3)).apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn apply_substring_reverse_from_over() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::From(4)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_reverse_to_below() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::To(0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_reverse_to_last_but_one() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::To(3)).apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn apply_substring_reverse_to_last() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::To(4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_to_over() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::To(5)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_from_first_to_below() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::FromTo(0, 0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_reverse_from_first_to_last_but_one() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::FromTo(0, 3)).apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn apply_substring_reverse_from_first_to_last() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::FromTo(0, 4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_reverse_from_last_to_last() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::FromTo(3, 4)).apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn apply_substring_reverse_from_last_to_over() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::FromTo(3, 5)).apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn apply_substring_reverse_from_over_to_over() {
        let mut string = String::from("ábčd");
        string = Filter::SubstringReverse(Range::FromTo(4, 5)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_replace_first() {
        let mut string = String::from("abcd_abcd");
        string = Filter::ReplaceFirst(Substitution {
            value: String::from("ab"),
            replacement: String::from("x"),
        })
        .apply(string);
        assert_eq!(string, "xcd_abcd");
    }

    #[test]
    fn apply_replace_all() {
        let mut string = String::from("abcd_abcd");
        string = Filter::ReplaceAll(Substitution {
            value: String::from("ab"),
            replacement: String::from("x"),
        })
        .apply(string);
        assert_eq!(string, "xcd_xcd");
    }

    #[test]
    fn apply_remove_first() {
        let mut string = String::from("abcd_abcd");
        string = Filter::ReplaceFirst(Substitution {
            value: String::from("ab"),
            replacement: String::new(),
        })
        .apply(string);
        assert_eq!(string, "cd_abcd");
    }

    #[test]
    fn apply_remove_all() {
        let mut string = String::from("abcd_abcd");
        string = Filter::ReplaceAll(Substitution {
            value: String::from("ab"),
            replacement: String::new(),
        })
        .apply(string);
        assert_eq!(string, "cd_cd");
    }

    #[test]
    fn apply_trim_none() {
        let mut string = String::from("abcd");
        string = Filter::Trim.apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn apply_trim() {
        let mut string = String::from(" abcd ");
        string = Filter::Trim.apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn apply_to_lowercase() {
        let mut string = String::from("ábčdÁBČD");
        string = Filter::ToLowercase.apply(string);
        assert_eq!(string, "ábčdábčd");
    }

    #[test]
    fn apply_to_uppercase() {
        let mut string = String::from("ábčdÁBČD");
        string = Filter::ToUppercase.apply(string);
        assert_eq!(string, "ÁBČDÁBČD");
    }

    #[test]
    fn apply_to_ascii() {
        let mut string = String::from("ábčdÁBČD");
        string = Filter::ToAscii.apply(string);
        assert_eq!(string, "abcdABCD");
    }

    #[test]
    fn apply_remove_non_ascii() {
        let mut string = String::from("ábčdÁBČD");
        string = Filter::RemoveNonAscii.apply(string);
        assert_eq!(string, "bdBD");
    }

    #[test]
    fn apply_left_pad_all() {
        let mut string = String::from("");
        string = Filter::LeftPad(String::from("0123")).apply(string);
        assert_eq!(string, "0123");
    }

    #[test]
    fn apply_left_pad_some() {
        let mut string = String::from("ab");
        string = Filter::LeftPad(String::from("0123")).apply(string);
        assert_eq!(string, "01ab");
    }

    #[test]
    fn apply_left_pad_none() {
        let mut string = String::from("abcd");
        string = Filter::LeftPad(String::from("0123")).apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn apply_right_pad_all() {
        let mut string = String::from("");
        string = Filter::RightPad(String::from("0123")).apply(string);
        assert_eq!(string, "0123");
    }

    #[test]
    fn apply_right_pad_some() {
        let mut string = String::from("ab");
        string = Filter::RightPad(String::from("0123")).apply(string);
        assert_eq!(string, "ab23");
    }

    #[test]
    fn apply_right_pad_none() {
        let mut string = String::from("abcd");
        string = Filter::RightPad(String::from("0123")).apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn apply_default_used() {
        let mut string = String::from("");
        string = Filter::Default(String::from("xyz")).apply(string);
        assert_eq!(string, "xyz");
    }

    #[test]
    fn apply_default_unused() {
        let mut string = String::from("abcd");
        string = Filter::Default(String::from("xyz")).apply(string);
        assert_eq!(string, "abcd");
    }
}
