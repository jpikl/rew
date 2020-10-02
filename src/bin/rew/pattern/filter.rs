use crate::pattern::char::{AsChar, Char};
use crate::pattern::eval;
use crate::pattern::parse;
use crate::pattern::range::Range;
use crate::pattern::reader::Reader;
use crate::pattern::regex::{add_capture_group_brackets, RegexHolder};
use crate::pattern::substitution::Substitution;
use std::fmt;
use unidecode::unidecode;

#[derive(Debug, PartialEq)]
pub enum Filter {
    Substring(Range),
    SubstringReverse(Range),
    ReplaceFirst(Substitution<String>),
    ReplaceAll(Substitution<String>),
    RegexReplaceFirst(Substitution<RegexHolder>),
    RegexReplaceAll(Substitution<RegexHolder>),
    Trim,
    ToLowercase,
    ToUppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(String),
    RightPad(String),
    Default(String),
    // TODO ExternalCommand
    //  - Passed through cli option: `--extern='realpath {}' --extern='tr a b'`
    //  - Filter references its number: `{p|e1|e2}`.
    //  - Modes:
    //    a) Spawn for every path, pass value as its argument: `realpath {}`.
    //    b) Spawn once, values paths separated by LF or NUL through its stdin/stdout: `tr a b`.
    //  - Mode a) is chosen over b) when `{}` value placeholder is used within command.
}

impl Filter {
    pub fn parse(reader: &mut Reader<Char>) -> parse::Result<Self> {
        let position = reader.position();

        if let Some(char) = reader.read() {
            match char.as_char() {
                'n' => Ok(Filter::Substring(Range::parse(reader)?)),
                'N' => Ok(Filter::SubstringReverse(Range::parse(reader)?)),
                'r' => Ok(Filter::ReplaceFirst(Substitution::parse_string(reader)?)),
                'R' => Ok(Filter::ReplaceAll(Substitution::parse_string(reader)?)),
                's' => Ok(Filter::RegexReplaceFirst(Substitution::parse_regex(
                    reader,
                )?)),
                'S' => Ok(Filter::RegexReplaceAll(Substitution::parse_regex(reader)?)),
                't' => Ok(Filter::Trim),
                'l' => Ok(Filter::ToLowercase),
                'u' => Ok(Filter::ToUppercase),
                'a' => Ok(Filter::ToAscii),
                'A' => Ok(Filter::RemoveNonAscii),
                '<' => Ok(Filter::LeftPad(Char::join(reader.read_to_end()))),
                '>' => Ok(Filter::RightPad(Char::join(reader.read_to_end()))),
                '?' => Ok(Filter::Default(Char::join(reader.read_to_end()))),
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
            Self::Substring(range) => {
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

            Self::SubstringReverse(range) => {
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

            Self::ReplaceFirst(Substitution { value, replacement }) => {
                Ok(string.replacen(value, replacement, 1))
            }

            Self::ReplaceAll(Substitution { value, replacement }) => {
                Ok(string.replace(value, replacement))
            }

            Self::RegexReplaceFirst(Substitution {
                value: RegexHolder(regex),
                replacement,
            }) => Ok(regex
                .replacen(&string, 1, add_capture_group_brackets(replacement).as_ref())
                .to_string()),

            Self::RegexReplaceAll(Substitution {
                value: RegexHolder(regex),
                replacement,
            }) => Ok(regex
                .replace_all(&string, add_capture_group_brackets(replacement).as_ref())
                .to_string()),

            Self::Trim => Ok(string.trim().to_string()),
            Self::ToLowercase => Ok(string.to_lowercase()),
            Self::ToUppercase => Ok(string.to_uppercase()),
            Self::ToAscii => Ok(unidecode(&string)),

            Self::RemoveNonAscii => {
                string.retain(|ch| ch.is_ascii());
                Ok(string)
            }

            Self::LeftPad(padding) => {
                for char in padding.chars().rev().skip(string.len()) {
                    string.insert(0, char);
                }
                Ok(string)
            }

            Self::RightPad(padding) => {
                for char in padding.chars().skip(string.len()) {
                    string.push(char);
                }
                Ok(string)
            }

            Self::Default(default) => {
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
            Self::Substring(range) => write!(formatter, "Substring {}", range),
            Self::SubstringReverse(range) => write!(formatter, "Substring (reverse) {}", range),
            Self::ReplaceFirst(substitution) => write!(formatter, "Replace first {}", substitution),
            Self::ReplaceAll(substitution) => write!(formatter, "Replace all {}", substitution),
            Self::RegexReplaceFirst(substitution) => write!(
                formatter,
                "Replace first regular expression {}",
                substitution
            ),
            Self::RegexReplaceAll(substitution) => write!(
                formatter,
                "Replace all regular expressions {}",
                substitution
            ),
            Self::Trim => write!(formatter, "Trim"),
            Self::ToLowercase => write!(formatter, "To lowercase"),
            Self::ToUppercase => write!(formatter, "To uppercase"),
            Self::ToAscii => write!(formatter, "To ASCII"),
            Self::RemoveNonAscii => write!(formatter, "Remove non-ASCII"),
            Self::LeftPad(padding) => write!(formatter, "Left pad with '{}'", padding),
            Self::RightPad(padding) => write!(formatter, "Right pad with '{}'", padding),
            Self::Default(default) => write!(formatter, "Use '{}' as default", default),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;
    use crate::utils::AnyString;
    use regex::Regex;

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
            parse("r/ab"),
            Ok(Filter::ReplaceFirst(Substitution {
                value: String::from("ab"),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("r/ab/cd"),
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
            parse("R/ab"),
            Ok(Filter::ReplaceAll(Substitution {
                value: String::from("ab"),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("R/ab/cd"),
            Ok(Filter::ReplaceAll(Substitution {
                value: String::from("ab"),
                replacement: String::from("cd"),
            })),
        );
    }

    #[test]
    fn parse_regex_replace_first() {
        assert_eq!(
            parse("s"),
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedSubstitution,
                range: 1..1,
            }),
        );
        assert_eq!(
            parse("s/[0-9]+"),
            Ok(Filter::RegexReplaceFirst(Substitution {
                value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("s/[0-9]+/cd"),
            Ok(Filter::RegexReplaceFirst(Substitution {
                value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                replacement: String::from("cd"),
            })),
        );
        assert_eq!(
            parse("s/[0-9+/cd"),
            Err(parse::Error {
                kind: parse::ErrorKind::SubstituteRegexInvalid(AnyString(String::from(
                    "This string is not compared by assertion"
                ))),
                range: 2..7,
            }),
        );
    }

    #[test]
    fn parse_regex_replace_all() {
        assert_eq!(
            parse("S"),
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedSubstitution,
                range: 1..1,
            }),
        );
        assert_eq!(
            parse("S/[0-9]+"),
            Ok(Filter::RegexReplaceAll(Substitution {
                value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("S/[0-9]+/cd"),
            Ok(Filter::RegexReplaceAll(Substitution {
                value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                replacement: String::from("cd"),
            })),
        );
        assert_eq!(
            parse("S/[0-9+/cd"),
            Err(parse::Error {
                kind: parse::ErrorKind::SubstituteRegexInvalid(AnyString(String::from(
                    "This string is not compared by assertion"
                ))),
                range: 2..7,
            }),
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
        // Each assert covers different evaluation branch
        assert_eq!(
            Filter::SubstringReverse(Range::FromTo(4, 5)).eval(String::from("ábčd")),
            Ok(String::from(""))
        );
        assert_eq!(
            Filter::SubstringReverse(Range::FromTo(5, 6)).eval(String::from("ábčd")),
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
    fn eval_regex_replace_first() {
        assert_eq!(
            Filter::RegexReplaceFirst(Substitution {
                value: RegexHolder(Regex::new("([0-9])([0-9]+)").unwrap()),
                replacement: String::from("_$2$1_"),
            })
            .eval(String::from("abc123def456")),
            Ok(String::from("abc_231_def456"))
        );
    }

    #[test]
    fn eval_regex_replace_all() {
        assert_eq!(
            Filter::RegexReplaceAll(Substitution {
                value: RegexHolder(Regex::new("([0-9])([0-9]+)").unwrap()),
                replacement: String::from("_$2$1_"),
            })
            .eval(String::from("abc123def456")),
            Ok(String::from("abc_231_def_564_"))
        );
    }

    #[test]
    fn eval_regex_remove_first() {
        assert_eq!(
            Filter::RegexReplaceFirst(Substitution {
                value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                replacement: String::new(),
            })
            .eval(String::from("abc123def456")),
            Ok(String::from("abcdef456"))
        );
    }

    #[test]
    fn eval_regex_remove_all() {
        assert_eq!(
            Filter::RegexReplaceAll(Substitution {
                value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                replacement: String::new(),
            })
            .eval(String::from("abc123def456")),
            Ok(String::from("abcdef"))
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

    #[test]
    fn fmt() {
        assert_eq!(
            Filter::Substring(Range::FromTo(1, 3)).to_string(),
            "Substring from 2 to 3"
        );
        assert_eq!(
            Filter::SubstringReverse(Range::FromTo(1, 3)).to_string(),
            "Substring (reverse) from 2 to 3"
        );
        assert_eq!(
            Filter::ReplaceFirst(Substitution {
                value: String::from("a"),
                replacement: String::from("b")
            })
            .to_string(),
            "Replace first 'a' by 'b'"
        );
        assert_eq!(
            Filter::ReplaceAll(Substitution {
                value: String::from("a"),
                replacement: String::from("b")
            })
            .to_string(),
            "Replace all 'a' by 'b'"
        );
        assert_eq!(
            Filter::RegexReplaceFirst(Substitution {
                value: RegexHolder(Regex::new("a+").unwrap()),
                replacement: String::from("b")
            })
            .to_string(),
            "Replace first regular expression 'a+' by 'b'"
        );
        assert_eq!(
            Filter::RegexReplaceAll(Substitution {
                value: RegexHolder(Regex::new("a+").unwrap()),
                replacement: String::from("b")
            })
            .to_string(),
            "Replace all regular expressions 'a+' by 'b'"
        );
        assert_eq!(Filter::Trim.to_string(), "Trim");
        assert_eq!(Filter::ToLowercase.to_string(), "To lowercase");
        assert_eq!(Filter::ToUppercase.to_string(), "To uppercase");
        assert_eq!(Filter::ToAscii.to_string(), "To ASCII");
        assert_eq!(Filter::RemoveNonAscii.to_string(), "Remove non-ASCII");
        assert_eq!(
            Filter::LeftPad(String::from("abc")).to_string(),
            "Left pad with 'abc'"
        );
        assert_eq!(
            Filter::RightPad(String::from("abc")).to_string(),
            "Right pad with 'abc'"
        );
        assert_eq!(
            Filter::Default(String::from("abc")).to_string(),
            "Use 'abc' as default"
        );
    }
}
