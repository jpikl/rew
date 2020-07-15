use crate::pattern::char::{AsChar, Char};
use crate::pattern::eval;
use crate::pattern::parse;
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
    pub fn parse(reader: &mut Reader<Char>) -> parse::Result<Self> {
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
                '?' => Ok(Filter::Default(Char::join(reader.read_to_end()))),
                // TODO 'e' ExternalCommand
                _ => Err(parse::Error {
                    kind: parse::ErrorKind::UnknownFilter(char.clone()),
                    range: position..reader.position(),
                }),
            }
        } else {
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedFilter,
                range: position..reader.end(),
            })
        }
    }

    pub fn eval(&self, mut string: String) -> Result<String, eval::ErrorKind> {
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
                Ok(string)
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
                Ok(string)
            }

            Filter::ReplaceFirst(Substitution { value, replacement }) => {
                Ok(string.replacen(value, replacement, 1))
            }

            Filter::ReplaceAll(Substitution { value, replacement }) => {
                Ok(string.replace(value, replacement))
            }

            Filter::Trim => Ok(string.trim().to_string()),
            Filter::ToLowercase => Ok(string.to_lowercase()),
            Filter::ToUppercase => Ok(string.to_uppercase()),
            Filter::ToAscii => Ok(unidecode(&string)),

            Filter::RemoveNonAscii => {
                string.retain(|ch| ch.is_ascii());
                Ok(string)
            }

            Filter::LeftPad(padding) => {
                for char in padding.chars().rev().skip(string.len()) {
                    string.insert(0, char);
                }
                Ok(string)
            }

            Filter::RightPad(padding) => {
                for char in padding.chars().skip(string.len()) {
                    string.push(char);
                }
                Ok(string)
            }

            Filter::Default(default) => {
                if string.is_empty() {
                    string.push_str(default);
                }
                Ok(string)
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
            Filter::Trim => write!(formatter, "Trim"),
            Filter::ToLowercase => write!(formatter, "To lowercase"),
            Filter::ToUppercase => write!(formatter, "To uppercase"),
            Filter::ToAscii => write!(formatter, "To ASCII"),
            Filter::RemoveNonAscii => write!(formatter, "Remove non-ASCII"),
            Filter::LeftPad(padding) => write!(formatter, "Left pad with '{}'", padding),
            Filter::RightPad(padding) => write!(formatter, "Right pad with '{}'", padding),
            Filter::Default(default) => write!(formatter, "Use '{}' as default", default),
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
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedRange,
                range: 1..1,
            }),
        );
        assert_eq!(parse("n5"), Ok(Filter::Substring(Range::FromTo(4, 5))));
        assert_eq!(parse("n2-10"), Ok(Filter::Substring(Range::FromTo(1, 10))));
        assert_eq!(parse("n2-"), Ok(Filter::Substring(Range::From(1))));
        assert_eq!(parse("n-10"), Ok(Filter::Substring(Range::To(10))));
    }

    #[test]
    fn parse_substring_from_end() {
        assert_eq!(
            parse("N"),
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedRange,
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
    }

    #[test]
    fn parse_replace_first() {
        assert_eq!(
            parse("r"),
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedSubstitution,
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
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedSubstitution,
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
        assert_eq!(parse("?abc"), Ok(Filter::Default(String::from("abc"))));
    }

    #[test]
    fn parse_default_empty() {
        assert_eq!(parse("?"), Ok(Filter::Default(String::new())));
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
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedFilter,
                range: 0..0,
            }),
        )
    }

    #[test]
    fn parse_unknown_filter_error() {
        assert_eq!(
            parse("-_"),
            Err(parse::Error {
                kind: parse::ErrorKind::UnknownFilter(Char::Raw('-')),
                range: 0..1,
            }),
        );
    }

    fn parse(string: &str) -> parse::Result<Filter> {
        Filter::parse(&mut Reader::from(string))
    }

    #[test]
    fn eval_substring_from_first() {
        assert_eq!(
            Filter::Substring(Range::From(0)).eval(String::from("ábčd")),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn eval_substring_from_last() {
        assert_eq!(
            Filter::Substring(Range::From(3)).eval(String::from("ábčd")),
            Ok(String::from("d"))
        );
    }

    #[test]
    fn eval_substring_from_over() {
        assert_eq!(
            Filter::Substring(Range::From(4)).eval(String::from("ábčd")),
            Ok(String::from(""))
        );
    }

    #[test]
    fn eval_substring_to_below() {
        assert_eq!(
            Filter::Substring(Range::To(0)).eval(String::from("ábčd")),
            Ok(String::from(""))
        );
    }

    #[test]
    fn eval_substring_to_last_but_one() {
        assert_eq!(
            Filter::Substring(Range::To(3)).eval(String::from("ábčd")),
            Ok(String::from("ábč"))
        );
    }

    #[test]
    fn eval_substring_to_last() {
        assert_eq!(
            Filter::Substring(Range::To(4)).eval(String::from("ábčd")),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn eval_substring_to_over() {
        assert_eq!(
            Filter::Substring(Range::To(5)).eval(String::from("ábčd")),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn eval_substring_from_first_to_below() {
        assert_eq!(
            Filter::Substring(Range::FromTo(0, 0)).eval(String::from("ábčd")),
            Ok(String::from(""))
        );
    }

    #[test]
    fn eval_substring_from_first_to_last_but_one() {
        assert_eq!(
            Filter::Substring(Range::FromTo(0, 3)).eval(String::from("ábčd")),
            Ok(String::from("ábč"))
        );
    }

    #[test]
    fn eval_substring_from_first_to_last() {
        assert_eq!(
            Filter::Substring(Range::FromTo(0, 4)).eval(String::from("ábčd")),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn eval_substring_from_last_to_last() {
        assert_eq!(
            Filter::Substring(Range::FromTo(3, 4)).eval(String::from("ábčd")),
            Ok(String::from("d"))
        );
    }

    #[test]
    fn eval_substring_from_last_to_over() {
        assert_eq!(
            Filter::Substring(Range::FromTo(3, 5)).eval(String::from("ábčd")),
            Ok(String::from("d"))
        );
    }

    #[test]
    fn eval_substring_from_over_to_over() {
        assert_eq!(
            Filter::Substring(Range::FromTo(4, 5)).eval(String::from("ábčd")),
            Ok(String::from(""))
        );
    }

    #[test]
    fn eval_substring_reverse_from_first() {
        assert_eq!(
            Filter::SubstringReverse(Range::From(0)).eval(String::from("ábčd")),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn eval_substring_reverse_from_last() {
        assert_eq!(
            Filter::SubstringReverse(Range::From(3)).eval(String::from("ábčd")),
            Ok(String::from("á"))
        );
    }

    #[test]
    fn eval_substring_reverse_from_over() {
        assert_eq!(
            Filter::SubstringReverse(Range::From(4)).eval(String::from("ábčd")),
            Ok(String::from(""))
        );
    }

    #[test]
    fn eval_substring_reverse_to_below() {
        assert_eq!(
            Filter::SubstringReverse(Range::To(0)).eval(String::from("ábčd")),
            Ok(String::from(""))
        );
    }

    #[test]
    fn eval_substring_reverse_to_last_but_one() {
        assert_eq!(
            Filter::SubstringReverse(Range::To(3)).eval(String::from("ábčd")),
            Ok(String::from("bčd"))
        );
    }

    #[test]
    fn eval_substring_reverse_to_last() {
        assert_eq!(
            Filter::SubstringReverse(Range::To(4)).eval(String::from("ábčd")),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn eval_substring_reverse_to_over() {
        assert_eq!(
            Filter::SubstringReverse(Range::To(5)).eval(String::from("ábčd")),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn eval_substring_reverse_from_first_to_below() {
        assert_eq!(
            Filter::SubstringReverse(Range::FromTo(0, 0)).eval(String::from("ábčd")),
            Ok(String::from(""))
        );
    }

    #[test]
    fn eval_substring_reverse_from_first_to_last_but_one() {
        assert_eq!(
            Filter::SubstringReverse(Range::FromTo(0, 3)).eval(String::from("ábčd")),
            Ok(String::from("bčd"))
        );
    }

    #[test]
    fn eval_substring_reverse_from_first_to_last() {
        assert_eq!(
            Filter::SubstringReverse(Range::FromTo(0, 4)).eval(String::from("ábčd")),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn eval_substring_reverse_from_last_to_last() {
        assert_eq!(
            Filter::SubstringReverse(Range::FromTo(3, 4)).eval(String::from("ábčd")),
            Ok(String::from("á"))
        );
    }

    #[test]
    fn eval_substring_reverse_from_last_to_over() {
        assert_eq!(
            Filter::SubstringReverse(Range::FromTo(3, 5)).eval(String::from("ábčd")),
            Ok(String::from("á"))
        );
    }

    #[test]
    fn eval_substring_reverse_from_over_to_over() {
        assert_eq!(
            Filter::SubstringReverse(Range::FromTo(4, 5)).eval(String::from("ábčd")),
            Ok(String::from(""))
        );
    }

    #[test]
    fn eval_replace_first() {
        assert_eq!(
            Filter::ReplaceFirst(Substitution {
                value: String::from("ab"),
                replacement: String::from("x"),
            })
            .eval(String::from("abcd_abcd")),
            Ok(String::from("xcd_abcd"))
        );
    }

    #[test]
    fn eval_replace_all() {
        assert_eq!(
            Filter::ReplaceAll(Substitution {
                value: String::from("ab"),
                replacement: String::from("x"),
            })
            .eval(String::from("abcd_abcd")),
            Ok(String::from("xcd_xcd"))
        );
    }

    #[test]
    fn eval_remove_first() {
        assert_eq!(
            Filter::ReplaceFirst(Substitution {
                value: String::from("ab"),
                replacement: String::new(),
            })
            .eval(String::from("abcd_abcd")),
            Ok(String::from("cd_abcd"))
        );
    }

    #[test]
    fn eval_remove_all() {
        assert_eq!(
            Filter::ReplaceAll(Substitution {
                value: String::from("ab"),
                replacement: String::new(),
            })
            .eval(String::from("abcd_abcd")),
            Ok(String::from("cd_cd"))
        );
    }

    #[test]
    fn eval_trim_none() {
        assert_eq!(
            Filter::Trim.eval(String::from("abcd")),
            Ok(String::from("abcd"))
        );
    }

    #[test]
    fn eval_trim() {
        assert_eq!(
            Filter::Trim.eval(String::from(" abcd ")),
            Ok(String::from("abcd"))
        );
    }

    #[test]
    fn eval_to_lowercase() {
        assert_eq!(
            Filter::ToLowercase.eval(String::from("ábčdÁBČD")),
            Ok(String::from("ábčdábčd"))
        );
    }

    #[test]
    fn eval_to_uppercase() {
        assert_eq!(
            Filter::ToUppercase.eval(String::from("ábčdÁBČD")),
            Ok(String::from("ÁBČDÁBČD"))
        );
    }

    #[test]
    fn eval_to_ascii() {
        assert_eq!(
            Filter::ToAscii.eval(String::from("ábčdÁBČD")),
            Ok(String::from("abcdABCD"))
        );
    }

    #[test]
    fn eval_remove_non_ascii() {
        assert_eq!(
            Filter::RemoveNonAscii.eval(String::from("ábčdÁBČD")),
            Ok(String::from("bdBD"))
        );
    }

    #[test]
    fn eval_left_pad_all() {
        assert_eq!(
            Filter::LeftPad(String::from("0123")).eval(String::from("")),
            Ok(String::from("0123"))
        );
    }

    #[test]
    fn eval_left_pad_some() {
        assert_eq!(
            Filter::LeftPad(String::from("0123")).eval(String::from("ab")),
            Ok(String::from("01ab"))
        );
    }

    #[test]
    fn eval_left_pad_none() {
        assert_eq!(
            Filter::LeftPad(String::from("0123")).eval(String::from("abcd")),
            Ok(String::from("abcd"))
        );
    }

    #[test]
    fn eval_right_pad_all() {
        assert_eq!(
            Filter::RightPad(String::from("0123")).eval(String::from("")),
            Ok(String::from("0123"))
        );
    }

    #[test]
    fn eval_right_pad_some() {
        assert_eq!(
            Filter::RightPad(String::from("0123")).eval(String::from("ab")),
            Ok(String::from("ab23"))
        );
    }

    #[test]
    fn eval_right_pad_none() {
        assert_eq!(
            Filter::RightPad(String::from("0123")).eval(String::from("abcd")),
            Ok(String::from("abcd"))
        );
    }

    #[test]
    fn eval_default_used() {
        assert_eq!(
            Filter::Default(String::from("xyz")).eval(String::from("")),
            Ok(String::from("xyz"))
        );
    }

    #[test]
    fn eval_default_unused() {
        assert_eq!(
            Filter::Default(String::from("xyz")).eval(String::from("abcd")),
            Ok(String::from("abcd"))
        );
    }
}
