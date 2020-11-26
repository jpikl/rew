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
    NormalizedPath,
    CanonicalPath,
    ParentDirectory,
    PathWithoutLastComponent,
    FileName,
    BaseName,
    PathWithoutExtension,
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
                'p' => Ok(Self::NormalizedPath),
                'P' => Ok(Self::CanonicalPath),
                'd' => Ok(Self::ParentDirectory),
                'D' => Ok(Self::PathWithoutLastComponent),
                'f' => Ok(Self::FileName),
                'b' => Ok(Self::BaseName),
                'B' => Ok(Self::PathWithoutExtension),
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
            Self::NormalizedPath => path::get_normalized(value),
            Self::CanonicalPath => path::get_canonical(value, context.current_dir),
            Self::ParentDirectory => path::get_parent_directory(value),
            Self::PathWithoutLastComponent => path::get_without_last_component(value),
            Self::FileName => path::get_file_name(value),
            Self::BaseName => path::get_base_name(value),
            Self::PathWithoutExtension => path::get_without_extension(value),
            Self::Extension => path::get_extension(value),
            Self::ExtensionWithDot => path::get_extension_with_dot(value),

            // Substring filters
            Self::Substring(range) => substr::get_forward(value, range.start(), range.length()),
            Self::SubstringBackward(range) => {
                substr::get_backward(value, range.start(), range.length())
            }

            // Replace filters
            Self::ReplaceFirst(subst) => {
                string::replace_first(value, &subst.target, &subst.replacement)
            }
            Self::ReplaceAll(subst) => {
                string::replace_all(value, &subst.target, &subst.replacement)
            }
            Self::ReplaceEmpty(replacement) => string::replace_empty(value, &replacement),

            // Regex filters
            Self::RegexMatch(RegexHolder(regex)) => regex::get_match(value, &regex),
            Self::RegexReplaceFirst(subst) => {
                regex::replace_first(value, &subst.target.0, &subst.replacement)
            }
            Self::RegexReplaceAll(subst) => {
                regex::replace_all(value, &subst.target.0, &subst.replacement)
            }
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
            Self::LeftPad(Padding::Repeated(repetition)) => {
                format::left_pad_repeat(value, &repetition.value, repetition.count)
            }
            Self::RightPad(Padding::Fixed(padding)) => format::right_pad(value, &padding),
            Self::RightPad(Padding::Repeated(repetition)) => {
                format::right_pad_repeat(value, &repetition.value, repetition.count)
            }

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
            Self::NormalizedPath => write!(formatter, "Normalized path"),
            Self::CanonicalPath => write!(formatter, "Canonical path"),
            Self::ParentDirectory => write!(formatter, "Parent directory"),
            Self::PathWithoutLastComponent => write!(formatter, "Path without last component"),
            Self::FileName => write!(formatter, "File name"),
            Self::BaseName => write!(formatter, "Base name"),
            Self::PathWithoutExtension => write!(formatter, "Path without extension"),
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
    extern crate regex;
    use super::Filter;
    use crate::pattern::char::Char;
    use crate::pattern::padding::Padding;
    use crate::pattern::range::{IndexRange, NumberInterval};
    use crate::pattern::regex::RegexHolder;
    use crate::pattern::repetition::Repetition;
    use crate::pattern::substitution::Substitution;
    use crate::utils::AnyString;
    use regex::Regex;

    mod parse {
        use super::*;
        use crate::pattern::parse::{Error, ErrorKind, Result};
        use crate::pattern::reader::Reader;

        #[test]
        fn empty() {
            assert_eq!(
                parse(""),
                Err(Error {
                    kind: ErrorKind::ExpectedFilter,
                    range: 0..0,
                }),
            )
        }

        #[test]
        fn unknown() {
            assert_eq!(
                parse("-a"),
                Err(Error {
                    kind: ErrorKind::UnknownFilter(Char::Raw('-')),
                    range: 0..1,
                }),
            );
        }

        #[test]
        fn chars_after() {
            let mut reader = Reader::from("a_");
            Filter::parse(&mut reader).unwrap();
            assert_eq!(reader.position(), 1);
        }

        #[test]
        fn absolute_path() {
            assert_eq!(parse("a"), Ok(Filter::AbsolutePath));
        }

        #[test]
        fn normalized_path() {
            assert_eq!(parse("p"), Ok(Filter::NormalizedPath));
        }

        #[test]
        fn canonical_path() {
            assert_eq!(parse("P"), Ok(Filter::CanonicalPath));
        }

        #[test]
        fn parent_directory() {
            assert_eq!(parse("d"), Ok(Filter::ParentDirectory));
        }

        #[test]
        fn path_without_filename() {
            assert_eq!(parse("D"), Ok(Filter::PathWithoutLastComponent));
        }

        #[test]
        fn file_name() {
            assert_eq!(parse("f"), Ok(Filter::FileName));
        }

        #[test]
        fn base_name() {
            assert_eq!(parse("b"), Ok(Filter::BaseName));
        }

        #[test]
        fn path_without_extension() {
            assert_eq!(parse("B"), Ok(Filter::PathWithoutExtension));
        }

        #[test]
        fn extension() {
            assert_eq!(parse("e"), Ok(Filter::Extension));
        }

        #[test]
        fn extension_with_dot() {
            assert_eq!(parse("E"), Ok(Filter::ExtensionWithDot));
        }

        #[test]
        fn substring() {
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
            assert_eq!(
                parse("n"),
                Err(Error {
                    kind: ErrorKind::ExpectedRange,
                    range: 1..1,
                }),
            );
        }

        #[test]
        fn substring_backward() {
            assert_eq!(
                parse("N"),
                Err(Error {
                    kind: ErrorKind::ExpectedRange,
                    range: 1..1,
                }),
            );
            assert_eq!(
                parse("N2"),
                Ok(Filter::SubstringBackward(IndexRange::new(1, Some(1))))
            );
            assert_eq!(
                parse("N2-"),
                Ok(Filter::SubstringBackward(IndexRange::new(1, None)))
            );
            assert_eq!(
                parse("N2-10"),
                Ok(Filter::SubstringBackward(IndexRange::new(1, Some(9))))
            );
        }

        #[test]
        fn replace_first() {
            assert_eq!(
                parse("r"),
                Err(Error {
                    kind: ErrorKind::ExpectedSubstitution,
                    range: 1..1,
                }),
            );
            assert_eq!(
                parse("r/ab"),
                Ok(Filter::ReplaceFirst(Substitution {
                    target: String::from("ab"),
                    replacement: String::new(),
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
        fn replace_all() {
            assert_eq!(
                parse("R"),
                Err(Error {
                    kind: ErrorKind::ExpectedSubstitution,
                    range: 1..1,
                }),
            );
            assert_eq!(
                parse("R/ab"),
                Ok(Filter::ReplaceAll(Substitution {
                    target: String::from("ab"),
                    replacement: String::new(),
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
        fn replace_empty() {
            assert_eq!(parse("?abc"), Ok(Filter::ReplaceEmpty(String::from("abc"))));
        }

        #[test]
        fn regex_match() {
            assert_eq!(
                parse("m"),
                Err(Error {
                    kind: ErrorKind::ExpectedRegex,
                    range: 1..1,
                }),
            );
            assert_eq!(
                parse("m[0-9]+"),
                Ok(Filter::RegexMatch(RegexHolder(
                    Regex::new("[0-9]+").unwrap()
                ))),
            );
            assert_eq!(
                parse("m[0-9+"),
                Err(Error {
                    kind: ErrorKind::RegexInvalid(AnyString(String::from(
                        "This string is not compared by assertion"
                    ))),
                    range: 1..6,
                }),
            );
        }

        #[test]
        fn regex_replace_first() {
            assert_eq!(
                parse("s"),
                Err(Error {
                    kind: ErrorKind::ExpectedSubstitution,
                    range: 1..1,
                }),
            );
            assert_eq!(
                parse("s/[0-9]+"),
                Ok(Filter::RegexReplaceFirst(Substitution {
                    target: RegexHolder(Regex::new("[0-9]+").unwrap()),
                    replacement: String::new(),
                })),
            );
            assert_eq!(
                parse("s/[0-9]+/cd"),
                Ok(Filter::RegexReplaceFirst(Substitution {
                    target: RegexHolder(Regex::new("[0-9]+").unwrap()),
                    replacement: String::from("cd"),
                })),
            );
            assert_eq!(
                parse("s/[0-9+/cd"),
                Err(Error {
                    kind: ErrorKind::SubstitutionRegexInvalid(AnyString(String::from(
                        "This string is not compared by assertion"
                    ))),
                    range: 2..7,
                }),
            );
        }

        #[test]
        fn regex_replace_all() {
            assert_eq!(
                parse("S"),
                Err(Error {
                    kind: ErrorKind::ExpectedSubstitution,
                    range: 1..1,
                }),
            );
            assert_eq!(
                parse("S/[0-9]+"),
                Ok(Filter::RegexReplaceAll(Substitution {
                    target: RegexHolder(Regex::new("[0-9]+").unwrap()),
                    replacement: String::new(),
                })),
            );
            assert_eq!(
                parse("S/[0-9]+/cd"),
                Ok(Filter::RegexReplaceAll(Substitution {
                    target: RegexHolder(Regex::new("[0-9]+").unwrap()),
                    replacement: String::from("cd"),
                })),
            );
            assert_eq!(
                parse("S/[0-9+/cd"),
                Err(Error {
                    kind: ErrorKind::SubstitutionRegexInvalid(AnyString(String::from(
                        "This string is not compared by assertion"
                    ))),
                    range: 2..7,
                }),
            );
        }

        #[test]
        fn regex_capture() {
            assert_eq!(
                parse("0"),
                Err(Error {
                    kind: ErrorKind::RegexCaptureZero,
                    range: 0..1,
                }),
            );
            assert_eq!(parse("1"), Ok(Filter::RegexCapture(1)));
            assert_eq!(parse("2"), Ok(Filter::RegexCapture(2)));
            assert_eq!(parse("10"), Ok(Filter::RegexCapture(10)));
        }

        #[test]
        fn trim() {
            assert_eq!(parse("t"), Ok(Filter::Trim));
        }

        #[test]
        fn to_lowercase() {
            assert_eq!(parse("l"), Ok(Filter::ToLowercase));
        }

        #[test]
        fn to_uppercase() {
            assert_eq!(parse("L"), Ok(Filter::ToUppercase));
        }

        #[test]
        fn to_ascii() {
            assert_eq!(parse("i"), Ok(Filter::ToAscii));
        }

        #[test]
        fn remove_non_ascii() {
            assert_eq!(parse("I"), Ok(Filter::RemoveNonAscii));
        }

        #[test]
        fn left_pad() {
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
        fn right_pad() {
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
        fn repeat() {
            assert_eq!(
                parse("*3:abc"),
                Ok(Filter::Repeat(Repetition {
                    count: 3,
                    value: String::from("abc")
                }))
            );
        }

        #[test]
        fn local_counter() {
            assert_eq!(parse("c"), Ok(Filter::LocalCounter));
        }

        #[test]
        fn global_counter() {
            assert_eq!(parse("C"), Ok(Filter::GlobalCounter));
        }

        #[test]
        fn random_number() {
            assert_eq!(
                parse("u1-10"),
                Ok(Filter::RandomNumber(NumberInterval::new(1, Some(10))))
            );
        }

        #[test]
        fn random_uuid() {
            assert_eq!(parse("U"), Ok(Filter::RandomUuid));
        }

        fn parse(string: &str) -> Result<Filter> {
            Filter::parse(&mut Reader::from(string))
        }
    }

    mod eval {
        use super::*;
        use crate::pattern::filter::testing::assert_ok_uuid;
        use crate::pattern::testing::make_eval_context;
        use std::path::MAIN_SEPARATOR;

        #[test]
        fn absolute_path() {
            assert_eq!(
                Filter::AbsolutePath
                    .eval(String::from("root/parent/file.ext"), &make_eval_context()),
                Ok(format!("current_dir{}root/parent/file.ext", MAIN_SEPARATOR))
            );
        }

        #[test]
        fn normalized_path() {
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
        fn canonical_path() {
            let current_dir = std::env::current_dir().unwrap();
            let mut context = make_eval_context();
            context.current_dir = &current_dir;

            assert_eq!(
                Filter::CanonicalPath.eval(String::from("Cargo.toml"), &context),
                Ok(current_dir.join("Cargo.toml").to_str().unwrap().to_string())
            );
        }

        #[test]
        fn parent_directory() {
            assert_eq!(
                Filter::ParentDirectory
                    .eval(String::from("root/parent/file.ext"), &make_eval_context()),
                Ok(String::from("root/parent"))
            );
        }

        #[test]
        fn path_without_filename() {
            assert_eq!(
                Filter::PathWithoutLastComponent
                    .eval(String::from("root/parent/file.ext"), &make_eval_context()),
                Ok(String::from("root/parent"))
            );
        }

        #[test]
        fn file_name() {
            assert_eq!(
                Filter::FileName.eval(String::from("root/parent/file.ext"), &make_eval_context()),
                Ok(String::from("file.ext"))
            );
        }

        #[test]
        fn base_name() {
            assert_eq!(
                Filter::BaseName.eval(String::from("root/parent/file.ext"), &make_eval_context()),
                Ok(String::from("file"))
            );
        }

        #[test]
        fn path_without_extension() {
            assert_eq!(
                Filter::PathWithoutExtension
                    .eval(String::from("root/parent/file.ext"), &make_eval_context()),
                Ok(String::from("root/parent/file"))
            );
        }

        #[test]
        fn extension() {
            assert_eq!(
                Filter::Extension.eval(String::from("root/parent/file.ext"), &make_eval_context()),
                Ok(String::from("ext"))
            );
        }

        #[test]
        fn extension_with_dot() {
            assert_eq!(
                Filter::ExtensionWithDot
                    .eval(String::from("root/parent/file.ext"), &make_eval_context()),
                Ok(String::from(".ext"))
            );
        }

        #[test]
        fn substring() {
            assert_eq!(
                Filter::Substring(IndexRange::new(1, Some(2)))
                    .eval(String::from("abcde"), &make_eval_context()),
                Ok(String::from("bc"))
            );
        }

        #[test]
        fn substring_backward() {
            assert_eq!(
                Filter::SubstringBackward(IndexRange::new(1, Some(2)))
                    .eval(String::from("abcde"), &make_eval_context()),
                Ok(String::from("cd"))
            );
        }

        #[test]
        fn replace_first() {
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
        fn replace_all() {
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
        fn replace_empty() {
            assert_eq!(
                Filter::ReplaceEmpty(String::from("xyz")).eval(String::new(), &make_eval_context()),
                Ok(String::from("xyz"))
            );
        }

        #[test]
        fn regex_match() {
            assert_eq!(
                Filter::RegexMatch(RegexHolder(Regex::new("\\d+").unwrap()))
                    .eval(String::from("a123y"), &make_eval_context()),
                Ok(String::from("123"))
            );
        }

        #[test]
        fn regex_replace_first() {
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
        fn regex_replace_all() {
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
        fn regex_capture() {
            assert_eq!(
                Filter::RegexCapture(1).eval(String::new(), &make_eval_context()),
                Ok(String::from("abc"))
            );
        }

        #[test]
        fn trim() {
            assert_eq!(
                Filter::Trim.eval(String::from(" abcd "), &make_eval_context()),
                Ok(String::from("abcd"))
            );
        }

        #[test]
        fn to_lowercase() {
            assert_eq!(
                Filter::ToLowercase.eval(String::from("ábčdÁBČD"), &make_eval_context()),
                Ok(String::from("ábčdábčd"))
            );
        }

        #[test]
        fn to_uppercase() {
            assert_eq!(
                Filter::ToUppercase.eval(String::from("ábčdÁBČD"), &make_eval_context()),
                Ok(String::from("ÁBČDÁBČD"))
            );
        }

        #[test]
        fn to_ascii() {
            assert_eq!(
                Filter::ToAscii.eval(String::from("ábčdÁBČD"), &make_eval_context()),
                Ok(String::from("abcdABCD"))
            );
        }

        #[test]
        fn remove_non_ascii() {
            assert_eq!(
                Filter::RemoveNonAscii.eval(String::from("ábčdÁBČD"), &make_eval_context()),
                Ok(String::from("bdBD"))
            );
        }

        #[test]
        fn left_pad() {
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
        fn right_pad() {
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
        fn repeat() {
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
        fn local_counter() {
            assert_eq!(
                Filter::LocalCounter.eval(String::new(), &make_eval_context()),
                Ok(String::from("1"))
            );
        }

        #[test]
        fn global_counter() {
            assert_eq!(
                Filter::GlobalCounter.eval(String::new(), &make_eval_context()),
                Ok(String::from("2"))
            );
        }

        #[test]
        fn random_number() {
            assert_eq!(
                Filter::RandomNumber(NumberInterval::new(0, Some(0)))
                    .eval(String::new(), &make_eval_context()),
                Ok(String::from("0"))
            );
        }

        #[test]
        fn random_uuid() {
            assert_ok_uuid(Filter::RandomUuid.eval(String::new(), &make_eval_context()));
        }
    }

    mod display {
        use super::*;

        #[test]
        fn absolute_path() {
            assert_eq!(Filter::AbsolutePath.to_string(), "Absolute path");
        }

        #[test]
        fn normalized_path() {
            assert_eq!(Filter::NormalizedPath.to_string(), "Normalized path");
        }

        #[test]
        fn canonical_path() {
            assert_eq!(Filter::CanonicalPath.to_string(), "Canonical path");
        }

        #[test]
        fn parent_directory() {
            assert_eq!(Filter::ParentDirectory.to_string(), "Parent directory");
        }

        #[test]
        fn path_without_filename() {
            assert_eq!(
                Filter::PathWithoutLastComponent.to_string(),
                "Path without last component"
            );
        }

        #[test]
        fn file_name() {
            assert_eq!(Filter::FileName.to_string(), "File name");
        }

        #[test]
        fn base_name() {
            assert_eq!(Filter::BaseName.to_string(), "Base name");
        }

        #[test]
        fn path_without_extension() {
            assert_eq!(
                Filter::PathWithoutExtension.to_string(),
                "Path without extension"
            );
        }

        #[test]
        fn extension() {
            assert_eq!(Filter::Extension.to_string(), "Extension");
        }

        #[test]
        fn extension_with_dor() {
            assert_eq!(Filter::ExtensionWithDot.to_string(), "Extension with dot");
        }

        #[test]
        fn substring() {
            assert_eq!(
                Filter::Substring(IndexRange::new(1, Some(2))).to_string(),
                "Substring from 2..3"
            );
        }

        #[test]
        fn substring_backward() {
            assert_eq!(
                Filter::SubstringBackward(IndexRange::new(1, Some(2))).to_string(),
                "Substring (backward) from 2..3"
            );
        }

        #[test]
        fn replace_first() {
            assert_eq!(
                Filter::ReplaceFirst(Substitution {
                    target: String::from("a"),
                    replacement: String::from("b")
                })
                .to_string(),
                "Replace first 'a' with 'b'"
            );
        }

        #[test]
        fn replace_all() {
            assert_eq!(
                Filter::ReplaceAll(Substitution {
                    target: String::from("a"),
                    replacement: String::from("b")
                })
                .to_string(),
                "Replace all 'a' with 'b'"
            );
        }

        #[test]
        fn replace_empty() {
            assert_eq!(
                Filter::ReplaceEmpty(String::from("abc")).to_string(),
                "Replace empty with 'abc'"
            );
        }

        #[test]
        fn regex_match() {
            assert_eq!(
                Filter::RegexMatch(RegexHolder(Regex::new("a+").unwrap())).to_string(),
                "Match of regular expression 'a+'"
            );
        }

        #[test]
        fn regex_replace_first() {
            assert_eq!(
                Filter::RegexReplaceFirst(Substitution {
                    target: RegexHolder(Regex::new("a+").unwrap()),
                    replacement: String::from("b")
                })
                .to_string(),
                "Replace first match of regular expression 'a+' with 'b'"
            );
        }

        #[test]
        fn regex_replace_all() {
            assert_eq!(
                Filter::RegexReplaceAll(Substitution {
                    target: RegexHolder(Regex::new("a+").unwrap()),
                    replacement: String::from("b")
                })
                .to_string(),
                "Replace all matches of regular expression 'a+' with 'b'"
            );
        }

        #[test]
        fn regex_capture() {
            assert_eq!(
                Filter::RegexCapture(1).to_string(),
                "Regular expression capture #1"
            );
        }

        #[test]
        fn trim() {
            assert_eq!(Filter::Trim.to_string(), "Trim");
        }

        #[test]
        fn to_lowercase() {
            assert_eq!(Filter::ToLowercase.to_string(), "To lowercase");
        }

        #[test]
        fn to_uppercase() {
            assert_eq!(Filter::ToUppercase.to_string(), "To uppercase");
        }

        #[test]
        fn to_ascii() {
            assert_eq!(Filter::ToAscii.to_string(), "To ASCII");
        }

        #[test]
        fn remove_non_ascii() {
            assert_eq!(Filter::RemoveNonAscii.to_string(), "Remove non-ASCII");
        }

        #[test]
        fn left_pad() {
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
        }

        #[test]
        fn right_pad() {
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
        }

        #[test]
        fn repeat() {
            assert_eq!(
                Filter::Repeat(Repetition {
                    count: 5,
                    value: String::from("abc")
                })
                .to_string(),
                "Repeat 5x 'abc'"
            );
        }

        #[test]
        fn local_counter() {
            assert_eq!(Filter::LocalCounter.to_string(), "Local counter");
        }

        #[test]
        fn global_counter() {
            assert_eq!(Filter::GlobalCounter.to_string(), "Global counter");
        }

        #[test]
        fn random_number() {
            assert_eq!(
                Filter::RandomNumber(NumberInterval::new(0, Some(99))).to_string(),
                "Random number from [0, 99]"
            );
        }

        #[test]
        fn random_uuid() {
            assert_eq!(Filter::RandomUuid.to_string(), "Random UUID");
        }
    }
}
