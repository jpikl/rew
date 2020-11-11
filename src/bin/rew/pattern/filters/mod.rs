use crate::pattern::char::{AsChar, Char};
use crate::pattern::range::Range;
use crate::pattern::reader::Reader;
use crate::pattern::regex::RegexHolder;
use crate::pattern::substitution::Substitution;
use crate::pattern::{eval, parse};
use std::fmt;

mod error;
mod path;
mod substr;

#[derive(Debug, PartialEq)]
pub enum Filter {
    AbsolutePath,
    CanonicalPath,
    ParentPath,
    FileName,
    BaseName,
    Extension,
    ExtensionWithDot,
    Substring(Range),
    SubstringBackward(Range),
    ReplaceEmpty(String),
    ReplaceFirst(Substitution<String>),
    ReplaceAll(Substitution<String>),
    RegexMatch(RegexHolder),
    RegexReplaceFirst(Substitution<RegexHolder>),
    RegexReplaceAll(Substitution<RegexHolder>),
    Trim,
    ToLowercase,
    ToUppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(String),
    RightPad(String),
    LocalCounter,
    GlobalCounter,
    Uuid,
}

impl Filter {
    pub fn parse(reader: &mut Reader<Char>) -> parse::Result<Self> {
        let position = reader.position();

        if let Some(char) = reader.read() {
            match char.as_char() {
                'a' => Ok(Filter::AbsolutePath),
                'A' => Ok(Filter::CanonicalPath),
                'd' => Ok(Filter::ParentPath),
                'f' => Ok(Filter::FileName),
                'b' => Ok(Filter::BaseName),
                'e' => Ok(Filter::Extension),
                'E' => Ok(Filter::ExtensionWithDot),
                'n' => Ok(Filter::Substring(Range::parse(reader)?)),
                'N' => Ok(Filter::SubstringBackward(Range::parse(reader)?)),
                '?' => Ok(Filter::ReplaceEmpty(Char::join(reader.read_to_end()))),
                'r' => Ok(Filter::ReplaceFirst(Substitution::parse_string(reader)?)),
                'R' => Ok(Filter::ReplaceAll(Substitution::parse_string(reader)?)),
                'm' => Ok(Filter::RegexMatch(RegexHolder::parse(reader)?)),
                's' => Ok(Filter::RegexReplaceFirst(Substitution::parse_regex(
                    reader,
                )?)),
                'S' => Ok(Filter::RegexReplaceAll(Substitution::parse_regex(reader)?)),
                't' => Ok(Filter::Trim),
                'l' => Ok(Filter::ToLowercase),
                'L' => Ok(Filter::ToUppercase),
                'i' => Ok(Filter::ToAscii),
                'I' => Ok(Filter::RemoveNonAscii),
                '<' => Ok(Filter::LeftPad(Char::join(reader.read_to_end()))),
                '>' => Ok(Filter::RightPad(Char::join(reader.read_to_end()))),
                'c' => Ok(Filter::LocalCounter),
                'C' => Ok(Filter::GlobalCounter),
                'u' => Ok(Filter::Uuid),
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

    pub fn eval(
        &self,
        mut value: String,
        context: &eval::Context,
    ) -> Result<String, eval::ErrorKind> {
        match self {
            Filter::AbsolutePath => path::get_absolute(value, context.current_dir),
            Filter::CanonicalPath => path::get_canonical(value, context.current_dir),
            Filter::ParentPath => path::get_parent(value),
            Filter::FileName => path::get_file_name(value),
            Filter::BaseName => path::get_base_name(value),
            Filter::Extension => path::get_extension(value),
            Filter::ExtensionWithDot => path::get_extension_with_dot(value),
            Filter::Substring(range) => substr::get_forward(value, &range),
            Filter::SubstringBackward(range) => substr::get_backward(value, &range),
            Filter::ReplaceEmpty(value) => unimplemented!(),
            Filter::ReplaceFirst(substitution) => unimplemented!(),
            Filter::ReplaceAll(substitution) => unimplemented!(),
            Filter::RegexMatch(regex) => unimplemented!(),
            Filter::RegexReplaceFirst(substitution) => unimplemented!(),
            Filter::RegexReplaceAll(substitution) => unimplemented!(),
            Filter::Trim => unimplemented!(),
            Filter::ToLowercase => unimplemented!(),
            Filter::ToUppercase => unimplemented!(),
            Filter::ToAscii => unimplemented!(),
            Filter::RemoveNonAscii => unimplemented!(),
            Filter::LeftPad(String) => unimplemented!(),
            Filter::RightPad(String) => unimplemented!(),
            Filter::LocalCounter => unimplemented!(),
            Filter::GlobalCounter => unimplemented!(),
            Filter::Uuid => unimplemented!(),
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AbsolutePath => write!(formatter, "Absolute path"),
            Self::CanonicalPath => write!(formatter, "Canonical path"),
            Self::ParentPath => write!(formatter, "Parent path"),
            Self::FileName => write!(formatter, "File name"),
            Self::BaseName => write!(formatter, "Base name"),
            Self::Extension => write!(formatter, "Extension"),
            Self::ExtensionWithDot => write!(formatter, "Extension with dot"),
            Self::Substring(range) => write!(formatter, "Substring {}", range),
            Self::SubstringBackward(range) => write!(formatter, "Substring (backward) {}", range),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::char::Char;
    use crate::pattern::testing::make_eval_context;
    use crate::testing::make_non_utf8_os_str;
    use regex::Regex;
    use std::path::Path;

    #[test]
    fn parse_absolute_path() {
        assert_eq!(parse("a"), Ok(Filter::AbsolutePath));
    }

    #[test]
    fn parse_canonical_path() {
        assert_eq!(parse("A"), Ok(Filter::CanonicalPath));
    }

    #[test]
    fn parse_parent_path() {
        assert_eq!(parse("d"), Ok(Filter::ParentPath));
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
        assert_eq!(parse("n5"), Ok(Filter::Substring(Range::FromTo(4, 5))));
        assert_eq!(parse("n2-10"), Ok(Filter::Substring(Range::FromTo(1, 10))));
        assert_eq!(parse("n2-"), Ok(Filter::Substring(Range::From(1))));
        assert_eq!(parse("n-10"), Ok(Filter::Substring(Range::To(10))));
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
            parse("N5"),
            Ok(Filter::SubstringBackward(Range::FromTo(4, 5)))
        );
        assert_eq!(
            parse("N2-10"),
            Ok(Filter::SubstringBackward(Range::FromTo(1, 10)))
        );
        assert_eq!(parse("N2-"), Ok(Filter::SubstringBackward(Range::From(1))));
        assert_eq!(parse("N-10"), Ok(Filter::SubstringBackward(Range::To(10))));
    }

    fn parse(string: &str) -> parse::Result<Filter> {
        Filter::parse(&mut Reader::from(string))
    }

    #[test]
    fn eval_absolute_path() {
        assert_eq!(
            Filter::AbsolutePath.eval(String::from("root/parent/file.ext"), &make_eval_context()),
            Ok(String::from("current_dir/root/parent/file.ext"))
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
    fn substring() {
        assert_eq!(
            Filter::Substring(Range::FromTo(1, 3))
                .eval(String::from("abcde"), &make_eval_context()),
            Ok(String::from("bc"))
        );
    }

    #[test]
    fn substring_backward() {
        assert_eq!(
            Filter::SubstringBackward(Range::FromTo(1, 3))
                .eval(String::from("abcde"), &make_eval_context()),
            Ok(String::from("cd"))
        );
    }

    #[test]
    fn fmt() {
        assert_eq!(Filter::AbsolutePath.to_string(), "Absolute path");
        assert_eq!(Filter::CanonicalPath.to_string(), "Canonical path");
        assert_eq!(Filter::ParentPath.to_string(), "Parent path");
        assert_eq!(Filter::FileName.to_string(), "File name");
        assert_eq!(Filter::BaseName.to_string(), "Base name");
        assert_eq!(Filter::Extension.to_string(), "Extension");
        assert_eq!(Filter::ExtensionWithDot.to_string(), "Extension with dot");
        assert_eq!(
            Filter::Substring(Range::FromTo(1, 3)).to_string(),
            "Substring from 2 to 3"
        );
        assert_eq!(
            Filter::SubstringBackward(Range::FromTo(1, 3)).to_string(),
            "Substring (backward) from 2 to 3"
        );
    }
}
