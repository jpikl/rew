use crate::pattern::char::{AsChar, Char};
use crate::pattern::number::parse_number;
use crate::pattern::padding::Padding;
use crate::pattern::range::{IndexRange, NumberInterval};
use crate::pattern::reader::Reader;
use crate::pattern::regex::RegexHolder;
use crate::pattern::repetition::Repetition;
use crate::pattern::substitution::Substitution;
use crate::pattern::{eval, parse};
use std::fmt;

mod error;
mod format;
mod generate;
mod path;
mod regex;
mod string;
mod substr;
#[cfg(test)]
mod testing;

#[derive(Debug, PartialEq)]
pub enum Filter {
    // Path filters
    AbsolutePath,
    CanonicalPath,
    NormalizedPath,
    ParentPath,
    FileName,
    BaseName,
    BaseNameWithPath,
    Extension,
    ExtensionWithDot,

    // Substring filters
    Substring(IndexRange),
    SubstringBackward(IndexRange),

    // Replace filters
    ReplaceFirst(Substitution<String>),
    ReplaceAll(Substitution<String>),
    ReplaceEmpty(String),

    // Regex filters
    RegexMatch(RegexHolder),
    RegexReplaceFirst(Substitution<RegexHolder>),
    RegexReplaceAll(Substitution<RegexHolder>),
    RegexCapture(usize),

    // Format filters
    Trim,
    ToLowercase,
    ToUppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(Padding),
    RightPad(Padding),

    // Generators
    Repeat(Repetition),
    LocalCounter,
    GlobalCounter,
    RandomNumber(NumberInterval),
    RandomUuid,
}

impl Filter {
    pub fn parse(reader: &mut Reader<Char>) -> parse::Result<Self> {
        let position = reader.position();

        if let Some('0'..='9') = reader.peek_char() {
            let number = parse_number(reader)?;
            if number > 0 {
                Ok(Filter::RegexCapture(number))
            } else {
                Err(parse::Error {
                    kind: parse::ErrorKind::RegexCaptureZero,
                    range: position..reader.position(),
                })
            }
        } else if let Some(char) = reader.read() {
            match char.as_char() {
                // Path filters
                'a' => Ok(Self::AbsolutePath),
                'A' => Ok(Self::CanonicalPath),
                'h' => Ok(Self::NormalizedPath),
                'p' => Ok(Self::ParentPath),
                'f' => Ok(Self::FileName),
                'b' => Ok(Self::BaseName),
                'B' => Ok(Self::BaseNameWithPath),
                'e' => Ok(Self::Extension),
                'E' => Ok(Self::ExtensionWithDot),

                // Substring filters
                'n' => Ok(Self::Substring(IndexRange::parse(reader)?)),
                'N' => Ok(Self::SubstringBackward(IndexRange::parse(reader)?)),

                // Replace filters
                'r' => Ok(Self::ReplaceFirst(Substitution::parse_string(reader)?)),
                'R' => Ok(Self::ReplaceAll(Substitution::parse_string(reader)?)),
                '?' => Ok(Self::ReplaceEmpty(Char::join(reader.read_to_end()))),

                // Regex filters
                'm' => Ok(Self::RegexMatch(RegexHolder::parse(reader)?)),
                's' => Ok(Self::RegexReplaceFirst(Substitution::parse_regex(reader)?)),
                'S' => Ok(Self::RegexReplaceAll(Substitution::parse_regex(reader)?)),

                // Format filters
                't' => Ok(Self::Trim),
                'l' => Ok(Self::ToLowercase),
                'L' => Ok(Self::ToUppercase),
                'i' => Ok(Self::ToAscii),
                'I' => Ok(Self::RemoveNonAscii),
                '<' => Ok(Self::LeftPad(Padding::parse(reader, '<')?)),
                '>' => Ok(Self::RightPad(Padding::parse(reader, '>')?)),

                // Generators
                '*' => Ok(Self::Repeat(Repetition::parse(reader)?)),
                'c' => Ok(Self::LocalCounter),
                'C' => Ok(Self::GlobalCounter),
                'u' => Ok(Self::RandomNumber(NumberInterval::parse(reader)?)),
                'U' => Ok(Self::RandomUuid),

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

    pub fn eval(&self, value: String, context: &eval::Context) -> Result<String, eval::ErrorKind> {
        match self {
            // Path filters
            Self::AbsolutePath => path::get_absolute(value, context.current_dir),
            Self::CanonicalPath => path::get_canonical(value, context.current_dir),
            Self::NormalizedPath => path::get_normalized(value),
            Self::ParentPath => path::get_parent_path(value),
            Self::FileName => path::get_file_name(value),
            Self::BaseName => path::get_base_name(value),
            Self::BaseNameWithPath => path::get_base_name_with_path(value),
            Self::Extension => path::get_extension(value),
            Self::ExtensionWithDot => path::get_extension_with_dot(value),

            // Substring filters
            Self::Substring(range) => substr::get_forward(value, range.start(), range.length()),
            Self::SubstringBackward(range) => {
                substr::get_backward(value, range.start(), range.length())
            }

            // Replace filters
            Self::ReplaceFirst(Substitution {
                target,
                replacement,
            }) => string::replace_first(value, &target, &replacement),

            Self::ReplaceAll(Substitution {
                target,
                replacement,
            }) => string::replace_all(value, &target, &replacement),
            Self::ReplaceEmpty(replacement) => string::replace_empty(value, &replacement),

            // Regex filters
            Self::RegexMatch(RegexHolder(regex)) => regex::get_match(value, &regex),

            Self::RegexReplaceFirst(Substitution {
                target: RegexHolder(regex),
                replacement,
            }) => regex::replace_first(value, &regex, &replacement),

            Self::RegexReplaceAll(Substitution {
                target: RegexHolder(regex),
                replacement,
            }) => regex::replace_all(value, &regex, &replacement),

            Self::RegexCapture(number) => {
                regex::get_capture(context.regex_captures.as_ref(), *number)
            }

            // Format filters
            Self::Trim => format::trim(value),
            Self::ToLowercase => format::to_lowercase(value),
            Self::ToUppercase => format::to_uppercase(value),
            Self::ToAscii => format::to_ascii(value),
            Self::RemoveNonAscii => format::remove_non_ascii(value),

            Self::LeftPad(Padding::Fixed(padding)) => format::left_pad(value, &padding),
            Self::LeftPad(Padding::Repeated(Repetition {
                value: padding,
                count,
            })) => format::left_pad_repeat(value, &padding, *count),

            Self::RightPad(Padding::Fixed(padding)) => format::right_pad(value, &padding),
            Self::RightPad(Padding::Repeated(Repetition {
                value: padding,
                count,
            })) => format::right_pad_repeat(value, &padding, *count),

            // Generators
            Self::Repeat(Repetition { value, count }) => generate::repetition(value, *count),
            Self::LocalCounter => generate::counter(context.local_counter),
            Self::GlobalCounter => generate::counter(context.global_counter),
            Self::RandomNumber(interval) => {
                generate::random_number(interval.start(), interval.end())
            }
            Self::RandomUuid => generate::random_uuid(),
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Path filters
            Self::AbsolutePath => write!(formatter, "Absolute path"),
            Self::CanonicalPath => write!(formatter, "Canonical path"),
            Self::NormalizedPath => write!(formatter, "Normalized path"),
            Self::ParentPath => write!(formatter, "Parent path"),
            Self::FileName => write!(formatter, "File name"),
            Self::BaseName => write!(formatter, "Base name"),
            Self::BaseNameWithPath => write!(formatter, "Base name with path"),
            Self::Extension => write!(formatter, "Extension"),
            Self::ExtensionWithDot => write!(formatter, "Extension with dot"),

            // Substring filters
            Self::Substring(range) => write!(formatter, "Substring from {}", range),
            Self::SubstringBackward(range) => {
                write!(formatter, "Substring (backward) from {}", range)
            }

            // Replace filters
            Self::ReplaceFirst(substitution) => write!(formatter, "Replace first {}", substitution),
            Self::ReplaceAll(substitution) => write!(formatter, "Replace all {}", substitution),
            Self::ReplaceEmpty(replacement) => {
                write!(formatter, "Replace empty with '{}'", replacement)
            }

            // Regex filters
            Self::RegexMatch(substitution) => {
                write!(formatter, "Match of regular expression '{}'", substitution)
            }
            Self::RegexReplaceFirst(substitution) => write!(
                formatter,
                "Replace first match of regular expression {}",
                substitution
            ),
            Self::RegexReplaceAll(substitution) => write!(
                formatter,
                "Replace all matches of regular expression {}",
                substitution
            ),
            Self::RegexCapture(number) => {
                write!(formatter, "Regular expression capture #{}", number)
            }

            // Format filters
            Self::Trim => write!(formatter, "Trim"),
            Self::ToLowercase => write!(formatter, "To lowercase"),
            Self::ToUppercase => write!(formatter, "To uppercase"),
            Self::ToAscii => write!(formatter, "To ASCII"),
            Self::RemoveNonAscii => write!(formatter, "Remove non-ASCII"),
            Self::LeftPad(padding) => write!(formatter, "Left pad with {}", padding),
            Self::RightPad(padding) => write!(formatter, "Right pad with {}", padding),

            // Generators
            Self::Repeat(repetition) => write!(formatter, "Repeat {}", repetition),
            Self::LocalCounter => write!(formatter, "Local counter"),
            Self::GlobalCounter => write!(formatter, "Global counter"),
            Self::RandomNumber(interval) => write!(formatter, "Random number from {}", interval),
            Self::RandomUuid => write!(formatter, "Random UUID"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::testing::make_eval_context;
    extern crate regex;
    use crate::pattern::filter::testing::assert_ok_uuid;
    use crate::pattern::parse::{Error, ErrorKind};
    use crate::utils::AnyString;
    use regex::Regex;
    use std::path::MAIN_SEPARATOR;

    #[test]
    fn parse_absolute_path() {
        assert_eq!(parse("a"), Ok(Filter::AbsolutePath));
    }

    #[test]
    fn parse_canonical_path() {
        assert_eq!(parse("A"), Ok(Filter::CanonicalPath));
    }

    #[test]
    fn parse_normalized_path() {
        assert_eq!(parse("h"), Ok(Filter::NormalizedPath));
    }

    #[test]
    fn parse_parent_path() {
        assert_eq!(parse("p"), Ok(Filter::ParentPath));
    }

    #[test]
    fn parse_file_name() {
        assert_eq!(parse("f"), Ok(Filter::FileName));
    }

    #[test]
    fn parse_base_name() {
        assert_eq!(parse("b"), Ok(Filter::BaseName));
    }

    #[test]
    fn parse_path_without_extension() {
        assert_eq!(parse("B"), Ok(Filter::BaseNameWithPath));
    }

    #[test]
    fn parse_extension() {
        assert_eq!(parse("e"), Ok(Filter::Extension));
    }

    #[test]
    fn parse_extension_with_dot() {
        assert_eq!(parse("E"), Ok(Filter::ExtensionWithDot));
    }

    #[test]
    fn parse_substring() {
        assert_eq!(
            parse("n"),
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedRange,
                range: 1..1,
            }),
        );
        assert_eq!(
            parse("n2-10"),
            Ok(Filter::Substring(IndexRange::new(1, Some(9))))
        );
        assert_eq!(
            parse("n2-"),
            Ok(Filter::Substring(IndexRange::new(1, None)))
        );
        assert_eq!(
            parse("n2"),
            Ok(Filter::Substring(IndexRange::new(1, Some(1))))
        );
    }

    #[test]
    fn parse_substring_backward() {
        assert_eq!(
            parse("N"),
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedRange,
                range: 1..1,
            }),
        );
        assert_eq!(
            parse("N2-10"),
            Ok(Filter::SubstringBackward(IndexRange::new(1, Some(9))))
        );
        assert_eq!(
            parse("N2-"),
            Ok(Filter::SubstringBackward(IndexRange::new(1, None)))
        );
        assert_eq!(
            parse("N2"),
            Ok(Filter::SubstringBackward(IndexRange::new(1, Some(1))))
        );
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
                target: String::from("ab"),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("r/ab/cd"),
            Ok(Filter::ReplaceFirst(Substitution {
                target: String::from("ab"),
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
                target: String::from("ab"),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("R/ab/cd"),
            Ok(Filter::ReplaceAll(Substitution {
                target: String::from("ab"),
                replacement: String::from("cd"),
            })),
        );
    }

    #[test]
    fn parse_replace_empty() {
        assert_eq!(parse("?abc"), Ok(Filter::ReplaceEmpty(String::from("abc"))));
    }

    #[test]
    fn parse_replace_empty_with_empty() {
        assert_eq!(parse("?"), Ok(Filter::ReplaceEmpty(String::new())));
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
        assert_eq!(parse("L"), Ok(Filter::ToUppercase));
    }

    #[test]
    fn parse_to_ascii() {
        assert_eq!(parse("i"), Ok(Filter::ToAscii));
    }

    #[test]
    fn parse_remove_non_ascii() {
        assert_eq!(parse("I"), Ok(Filter::RemoveNonAscii));
    }

    #[test]
    fn parse_left_pad() {
        assert_eq!(
            parse("<abc"),
            Err(Error {
                kind: ErrorKind::PaddingPrefixInvalid('<', Some(Char::Raw('a'))),
                range: 1..2
            })
        );
        assert_eq!(
            parse("<<abc"),
            Ok(Filter::LeftPad(Padding::Fixed(String::from("abc"))))
        );
        assert_eq!(
            parse("<10:abc"),
            Ok(Filter::LeftPad(Padding::Repeated(Repetition {
                count: 10,
                value: String::from("abc")
            })))
        );
    }

    #[test]
    fn parse_right_pad() {
        assert_eq!(
            parse(">abc"),
            Err(Error {
                kind: ErrorKind::PaddingPrefixInvalid('>', Some(Char::Raw('a'))),
                range: 1..2
            })
        );
        assert_eq!(
            parse(">>abc"),
            Ok(Filter::RightPad(Padding::Fixed(String::from("abc"))))
        );
        assert_eq!(
            parse(">10:abc"),
            Ok(Filter::RightPad(Padding::Repeated(Repetition {
                count: 10,
                value: String::from("abc")
            })))
        );
    }

    #[test]
    fn parse_regex_match() {
        assert_eq!(
            parse("m"),
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedRegex,
                range: 1..1,
            }),
        );
        assert_eq!(
            parse("m\\d+"),
            Ok(Filter::RegexMatch(RegexHolder(Regex::new("\\d+").unwrap()))),
        );
        assert_eq!(
            parse("m[0-9+"),
            Err(parse::Error {
                kind: parse::ErrorKind::RegexInvalid(AnyString(String::from(
                    "This string is not compared by assertion"
                ))),
                range: 1..6,
            }),
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
            parse("s/\\d+"),
            Ok(Filter::RegexReplaceFirst(Substitution {
                target: RegexHolder(Regex::new("\\d+").unwrap()),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("s/\\d+/cd"),
            Ok(Filter::RegexReplaceFirst(Substitution {
                target: RegexHolder(Regex::new("\\d+").unwrap()),
                replacement: String::from("cd"),
            })),
        );
        assert_eq!(
            parse("s/[0-9+/cd"),
            Err(parse::Error {
                kind: parse::ErrorKind::SubstitutionRegexInvalid(AnyString(String::from(
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
            parse("S/\\d+"),
            Ok(Filter::RegexReplaceAll(Substitution {
                target: RegexHolder(Regex::new("\\d+").unwrap()),
                replacement: String::from(""),
            })),
        );
        assert_eq!(
            parse("S/\\d+/cd"),
            Ok(Filter::RegexReplaceAll(Substitution {
                target: RegexHolder(Regex::new("\\d+").unwrap()),
                replacement: String::from("cd"),
            })),
        );
        assert_eq!(
            parse("S/[0-9+/cd"),
            Err(parse::Error {
                kind: parse::ErrorKind::SubstitutionRegexInvalid(AnyString(String::from(
                    "This string is not compared by assertion"
                ))),
                range: 2..7,
            }),
        );
    }

    #[test]
    fn parse_regex_capture() {
        assert_eq!(
            parse("0"),
            Err(parse::Error {
                kind: parse::ErrorKind::RegexCaptureZero,
                range: 0..1,
            }),
        );
        assert_eq!(parse("1"), Ok(Filter::RegexCapture(1)));
        assert_eq!(parse("2"), Ok(Filter::RegexCapture(2)));
        assert_eq!(parse("10"), Ok(Filter::RegexCapture(10)));
    }

    #[test]
    fn parse_repeat() {
        assert_eq!(
            parse("*3:abc"),
            Ok(Filter::Repeat(Repetition {
                count: 3,
                value: String::from("abc")
            }))
        );
    }

    #[test]
    fn parse_local_counter() {
        assert_eq!(parse("c"), Ok(Filter::LocalCounter));
    }

    #[test]
    fn parse_global_counter() {
        assert_eq!(parse("C"), Ok(Filter::GlobalCounter));
    }

    #[test]
    fn parse_random_number() {
        assert_eq!(
            parse("u1-10"),
            Ok(Filter::RandomNumber(NumberInterval::new(1, Some(10))))
        );
    }

    #[test]
    fn parse_random_uuid() {
        assert_eq!(parse("U"), Ok(Filter::RandomUuid));
    }

    #[test]
    fn parse_ignore_chars_after_filter() {
        let mut reader = Reader::from("a_");
        Filter::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 1);
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

    fn parse(string: &str) -> parse::Result<Filter> {
        Filter::parse(&mut Reader::from(string))
    }

    #[test]
    fn eval_absolute_path() {
        assert_eq!(
            Filter::AbsolutePath.eval(String::from("root/parent/file.ext"), &make_eval_context()),
            Ok(format!("current_dir{}root/parent/file.ext", MAIN_SEPARATOR))
        );
    }

    #[test]
    fn eval_canonical_path() {
        let current_dir = std::env::current_dir().unwrap();
        let mut context = make_eval_context();
        context.current_dir = &current_dir;

        assert_eq!(
            Filter::CanonicalPath.eval(String::from("Cargo.toml"), &context),
            Ok(current_dir.join("Cargo.toml").to_str().unwrap().to_string())
        );
    }

    #[test]
    fn eval_normalized_path() {
        assert_eq!(
            Filter::NormalizedPath.eval(
                String::from("root/parent/../new-parent/./dir/"),
                &make_eval_context()
            ),
            Ok(format!(
                "root{}new-parent{}dir",
                MAIN_SEPARATOR, MAIN_SEPARATOR
            ))
        );
    }

    #[test]
    fn eval_parent_path() {
        assert_eq!(
            Filter::ParentPath.eval(String::from("root/parent/file.ext"), &make_eval_context()),
            Ok(String::from("root/parent"))
        );
    }

    #[test]
    fn eval_file_name() {
        assert_eq!(
            Filter::FileName.eval(String::from("root/parent/file.ext"), &make_eval_context()),
            Ok(String::from("file.ext"))
        );
    }

    #[test]
    fn eval_base_name() {
        assert_eq!(
            Filter::BaseName.eval(String::from("root/parent/file.ext"), &make_eval_context()),
            Ok(String::from("file"))
        );
    }

    #[test]
    fn eval_base_name_with_path() {
        assert_eq!(
            Filter::BaseNameWithPath
                .eval(String::from("root/parent/file.ext"), &make_eval_context()),
            Ok(String::from("root/parent/file"))
        );
    }

    #[test]
    fn eval_extension() {
        assert_eq!(
            Filter::Extension.eval(String::from("root/parent/file.ext"), &make_eval_context()),
            Ok(String::from("ext"))
        );
    }

    #[test]
    fn eval_extension_with_dot() {
        assert_eq!(
            Filter::ExtensionWithDot
                .eval(String::from("root/parent/file.ext"), &make_eval_context()),
            Ok(String::from(".ext"))
        );
    }

    #[test]
    fn eval_substring() {
        assert_eq!(
            Filter::Substring(IndexRange::new(1, Some(2)))
                .eval(String::from("abcde"), &make_eval_context()),
            Ok(String::from("bc"))
        );
    }

    #[test]
    fn eval_substring_backward() {
        assert_eq!(
            Filter::SubstringBackward(IndexRange::new(1, Some(2)))
                .eval(String::from("abcde"), &make_eval_context()),
            Ok(String::from("cd"))
        );
    }

    #[test]
    fn eval_replace_first() {
        assert_eq!(
            Filter::ReplaceFirst(Substitution {
                target: String::from("ab"),
                replacement: String::from("x"),
            })
            .eval(String::from("abcd_abcd"), &make_eval_context()),
            Ok(String::from("xcd_abcd"))
        );
    }

    #[test]
    fn eval_replace_all() {
        assert_eq!(
            Filter::ReplaceAll(Substitution {
                target: String::from("ab"),
                replacement: String::from("x"),
            })
            .eval(String::from("abcd_abcd"), &make_eval_context()),
            Ok(String::from("xcd_xcd"))
        );
    }

    #[test]
    fn eval_replace_empty() {
        assert_eq!(
            Filter::ReplaceEmpty(String::from("xyz")).eval(String::new(), &make_eval_context()),
            Ok(String::from("xyz"))
        );
    }

    #[test]
    fn eval_regex_match() {
        assert_eq!(
            Filter::RegexMatch(RegexHolder(Regex::new("\\d+").unwrap()))
                .eval(String::from("a123y"), &make_eval_context()),
            Ok(String::from("123"))
        );
    }

    #[test]
    fn eval_regex_replace_first() {
        assert_eq!(
            Filter::RegexReplaceFirst(Substitution {
                target: RegexHolder(Regex::new("\\d+").unwrap()),
                replacement: String::from("x"),
            })
            .eval(String::from("12_34"), &make_eval_context()),
            Ok(String::from("x_34"))
        );
    }

    #[test]
    fn eval_regex_replace_all() {
        assert_eq!(
            Filter::RegexReplaceAll(Substitution {
                target: RegexHolder(Regex::new("\\d+").unwrap()),
                replacement: String::from("x"),
            })
            .eval(String::from("12_34"), &make_eval_context()),
            Ok(String::from("x_x"))
        );
    }

    #[test]
    fn eval_regex_capture() {
        assert_eq!(
            Filter::RegexCapture(1).eval(String::new(), &make_eval_context()),
            Ok(String::from("abc"))
        );
    }

    #[test]
    fn eval_trim() {
        assert_eq!(
            Filter::Trim.eval(String::from(" abcd "), &make_eval_context()),
            Ok(String::from("abcd"))
        );
    }

    #[test]
    fn eval_to_lowercase() {
        assert_eq!(
            Filter::ToLowercase.eval(String::from("ábčdÁBČD"), &make_eval_context()),
            Ok(String::from("ábčdábčd"))
        );
    }

    #[test]
    fn eval_to_uppercase() {
        assert_eq!(
            Filter::ToUppercase.eval(String::from("ábčdÁBČD"), &make_eval_context()),
            Ok(String::from("ÁBČDÁBČD"))
        );
    }

    #[test]
    fn eval_to_ascii() {
        assert_eq!(
            Filter::ToAscii.eval(String::from("ábčdÁBČD"), &make_eval_context()),
            Ok(String::from("abcdABCD"))
        );
    }

    #[test]
    fn eval_remove_non_ascii() {
        assert_eq!(
            Filter::RemoveNonAscii.eval(String::from("ábčdÁBČD"), &make_eval_context()),
            Ok(String::from("bdBD"))
        );
    }

    #[test]
    fn eval_left_pad() {
        assert_eq!(
            Filter::LeftPad(Padding::Fixed(String::from("0123")))
                .eval(String::from("ab"), &make_eval_context()),
            Ok(String::from("01ab"))
        );
        assert_eq!(
            Filter::LeftPad(Padding::Repeated(Repetition {
                count: 3,
                value: String::from("01")
            }))
            .eval(String::from("ab"), &make_eval_context()),
            Ok(String::from("0101ab"))
        );
    }

    #[test]
    fn eval_right_pad() {
        assert_eq!(
            Filter::RightPad(Padding::Fixed(String::from("0123")))
                .eval(String::from("ab"), &make_eval_context()),
            Ok(String::from("ab23"))
        );
        assert_eq!(
            Filter::RightPad(Padding::Repeated(Repetition {
                count: 3,
                value: String::from("01")
            }))
            .eval(String::from("ab"), &make_eval_context()),
            Ok(String::from("ab0101"))
        );
    }

    #[test]
    fn eval_repeat() {
        assert_eq!(
            Filter::Repeat(Repetition {
                count: 3,
                value: String::from("abc")
            })
            .eval(String::new(), &make_eval_context()),
            Ok(String::from("abcabcabc"))
        );
    }

    #[test]
    fn eval_local_counter() {
        assert_eq!(
            Filter::LocalCounter.eval(String::new(), &make_eval_context()),
            Ok(String::from("1"))
        );
    }

    #[test]
    fn eval_global_counter() {
        assert_eq!(
            Filter::GlobalCounter.eval(String::new(), &make_eval_context()),
            Ok(String::from("2"))
        );
    }

    #[test]
    fn eval_random_number() {
        assert_eq!(
            Filter::RandomNumber(NumberInterval::new(0, Some(0)))
                .eval(String::new(), &make_eval_context()),
            Ok(String::from("0"))
        );
    }

    #[test]
    fn eval_random_uuid() {
        assert_ok_uuid(Filter::RandomUuid.eval(String::new(), &make_eval_context()));
    }

    #[test]
    fn display() {
        assert_eq!(Filter::AbsolutePath.to_string(), "Absolute path");
        assert_eq!(Filter::CanonicalPath.to_string(), "Canonical path");
        assert_eq!(Filter::NormalizedPath.to_string(), "Normalized path");
        assert_eq!(Filter::ParentPath.to_string(), "Parent path");
        assert_eq!(Filter::FileName.to_string(), "File name");
        assert_eq!(Filter::BaseName.to_string(), "Base name");
        assert_eq!(Filter::BaseNameWithPath.to_string(), "Base name with path");
        assert_eq!(Filter::Extension.to_string(), "Extension");
        assert_eq!(Filter::ExtensionWithDot.to_string(), "Extension with dot");
        assert_eq!(
            Filter::Substring(IndexRange::new(1, Some(2))).to_string(),
            "Substring from 2..3"
        );
        assert_eq!(
            Filter::SubstringBackward(IndexRange::new(1, Some(2))).to_string(),
            "Substring (backward) from 2..3"
        );
        assert_eq!(
            Filter::ReplaceFirst(Substitution {
                target: String::from("a"),
                replacement: String::from("b")
            })
            .to_string(),
            "Replace first 'a' with 'b'"
        );
        assert_eq!(
            Filter::ReplaceAll(Substitution {
                target: String::from("a"),
                replacement: String::from("b")
            })
            .to_string(),
            "Replace all 'a' with 'b'"
        );
        assert_eq!(Filter::Trim.to_string(), "Trim");
        assert_eq!(Filter::ToLowercase.to_string(), "To lowercase");
        assert_eq!(Filter::ToUppercase.to_string(), "To uppercase");
        assert_eq!(Filter::ToAscii.to_string(), "To ASCII");
        assert_eq!(Filter::RemoveNonAscii.to_string(), "Remove non-ASCII");
        assert_eq!(
            Filter::LeftPad(Padding::Fixed(String::from("abc"))).to_string(),
            "Left pad with 'abc'"
        );
        assert_eq!(
            Filter::LeftPad(Padding::Repeated(Repetition {
                count: 5,
                value: String::from("abc")
            }))
            .to_string(),
            "Left pad with 5x 'abc'"
        );
        assert_eq!(
            Filter::RightPad(Padding::Fixed(String::from("abc"))).to_string(),
            "Right pad with 'abc'"
        );
        assert_eq!(
            Filter::RightPad(Padding::Repeated(Repetition {
                count: 5,
                value: String::from("abc")
            }))
            .to_string(),
            "Right pad with 5x 'abc'"
        );
        assert_eq!(
            Filter::ReplaceEmpty(String::from("abc")).to_string(),
            "Replace empty with 'abc'"
        );
        assert_eq!(
            Filter::RegexMatch(RegexHolder(Regex::new("a+").unwrap())).to_string(),
            "Match of regular expression 'a+'"
        );
        assert_eq!(
            Filter::RegexReplaceFirst(Substitution {
                target: RegexHolder(Regex::new("a+").unwrap()),
                replacement: String::from("b")
            })
            .to_string(),
            "Replace first match of regular expression 'a+' with 'b'"
        );
        assert_eq!(
            Filter::RegexReplaceAll(Substitution {
                target: RegexHolder(Regex::new("a+").unwrap()),
                replacement: String::from("b")
            })
            .to_string(),
            "Replace all matches of regular expression 'a+' with 'b'"
        );
        assert_eq!(
            Filter::RegexCapture(1).to_string(),
            "Regular expression capture #1"
        );
        assert_eq!(
            Filter::Repeat(Repetition {
                count: 5,
                value: String::from("abc")
            })
            .to_string(),
            "Repeat 5x 'abc'"
        );
        assert_eq!(Filter::LocalCounter.to_string(), "Local counter");
        assert_eq!(Filter::GlobalCounter.to_string(), "Global counter");
        assert_eq!(
            Filter::RandomNumber(NumberInterval::new(0, Some(99))).to_string(),
            "Random number from [0, 99]"
        );
        assert_eq!(Filter::RandomUuid.to_string(), "Random UUID");
    }
}
