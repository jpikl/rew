use crate::pattern::char::{AsChar, Char};
use crate::pattern::eval;
use crate::pattern::number::parse_usize;
use crate::pattern::parse;
use crate::pattern::reader::Reader;
use crate::utils::AnyString;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fmt, fs};
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum Variable {
    Path,
    AbsolutePath,
    CanonicalPath,
    FileName,
    BaseName,
    Extension,
    ExtensionWithDot,
    Parent,
    ParentFileName,
    LocalCounter,
    GlobalCounter,
    RegexCapture(usize),
    Uuid,
}

impl Variable {
    pub fn parse(reader: &mut Reader<Char>) -> parse::Result<Self> {
        let position = reader.position();

        if let Some('0'..='9') = reader.peek_char() {
            let number = parse_usize(reader)?;
            if number > 0 {
                Ok(Variable::RegexCapture(number))
            } else {
                Err(parse::Error {
                    kind: parse::ErrorKind::RegexCaptureZero,
                    range: position..reader.position(),
                })
            }
        } else if let Some(char) = reader.read() {
            match char.as_char() {
                'p' => Ok(Variable::Path),
                'a' => Ok(Variable::AbsolutePath),
                'A' => Ok(Variable::CanonicalPath),
                'f' => Ok(Variable::FileName),
                'b' => Ok(Variable::BaseName),
                'e' => Ok(Variable::Extension),
                'E' => Ok(Variable::ExtensionWithDot),
                'd' => Ok(Variable::Parent),
                'D' => Ok(Variable::ParentFileName),
                'c' => Ok(Variable::LocalCounter),
                'C' => Ok(Variable::GlobalCounter),
                'u' => Ok(Variable::Uuid),
                _ => Err(parse::Error {
                    kind: parse::ErrorKind::UnknownVariable(char.clone()),
                    range: position..reader.position(),
                }),
            }
        } else {
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedVariable,
                range: position..reader.end(),
            })
        }
    }

    pub fn eval(&self, context: &eval::Context) -> Result<String, eval::ErrorKind> {
        match self {
            Self::Path => to_string(context.path),
            Self::AbsolutePath => to_string(&get_absolute_path(context)),
            Self::CanonicalPath => match fs::canonicalize(&get_absolute_path(context)) {
                Ok(path) => to_string(&path),
                Err(error) => Err(eval::ErrorKind::CanonicalizationFailed(AnyString(
                    error.to_string(),
                ))),
            },
            Self::FileName => opt_to_string(context.path.file_name()),
            Self::BaseName => opt_to_string(context.path.file_stem()),
            Self::Extension => opt_to_string(context.path.extension()),

            Self::ExtensionWithDot => {
                let mut string = opt_to_string(context.path.extension())?;
                if !string.is_empty() {
                    string.insert(0, '.');
                }
                Ok(string)
            }

            Self::Parent => opt_to_string(context.path.parent()),
            Self::ParentFileName => opt_to_string(context.path.parent().and_then(Path::file_name)),
            Self::LocalCounter => Ok(context.local_counter.to_string()),
            Self::GlobalCounter => Ok(context.global_counter.to_string()),

            Self::RegexCapture(index) => Ok(context
                .regex_captures
                .as_ref()
                .and_then(|captures| captures.get(*index))
                .map(|capture| capture.as_str())
                .map_or_else(String::new, String::from)),

            Self::Uuid => {
                let mut buffer = Uuid::encode_buffer();
                let str = Uuid::new_v4().to_hyphenated().encode_lower(&mut buffer);
                Ok((*str).to_string())
            }
        }
    }
}

pub fn get_absolute_path(context: &eval::Context) -> PathBuf {
    if context.path.is_absolute() {
        context.path.to_path_buf()
    } else {
        context.current_dir.join(context.path)
    }
}

pub fn opt_to_string<S: AsRef<OsStr> + ?Sized>(
    value: Option<&S>,
) -> Result<String, eval::ErrorKind> {
    if let Some(value) = value {
        to_string(value)
    } else {
        Ok(String::new())
    }
}

pub fn to_string<S: AsRef<OsStr> + ?Sized>(value: &S) -> Result<String, eval::ErrorKind> {
    if let Some(str) = value.as_ref().to_str() {
        Ok(str.to_string())
    } else {
        Err(eval::ErrorKind::InputNotUtf8)
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Path => write!(formatter, "Path"),
            Self::AbsolutePath => write!(formatter, "Absolute path"),
            Self::CanonicalPath => write!(formatter, "Canonical path"),
            Self::FileName => write!(formatter, "File name"),
            Self::BaseName => write!(formatter, "Base name"),
            Self::Extension => write!(formatter, "Extension"),
            Self::ExtensionWithDot => write!(formatter, "Extension with dot"),
            Self::Parent => write!(formatter, "Parent"),
            Self::ParentFileName => write!(formatter, "Parent file name"),
            Self::LocalCounter => write!(formatter, "Local counter"),
            Self::GlobalCounter => write!(formatter, "Global counter"),
            Self::RegexCapture(index) => write!(formatter, "Regex capture ({})", index),
            Self::Uuid => write!(formatter, "UUID"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::char::Char;
    use regex::Regex;
    use std::path::Path;

    #[test]
    fn parse_path() {
        assert_eq!(parse("p"), Ok(Variable::Path));
    }

    #[test]
    fn parse_absolute_path() {
        assert_eq!(parse("a"), Ok(Variable::AbsolutePath));
    }

    #[test]
    fn parse_canonical_path() {
        assert_eq!(parse("A"), Ok(Variable::CanonicalPath));
    }

    #[test]
    fn parse_file_name() {
        assert_eq!(parse("f"), Ok(Variable::FileName));
    }

    #[test]
    fn parse_base_name() {
        assert_eq!(parse("b"), Ok(Variable::BaseName));
    }

    #[test]
    fn parse_extension() {
        assert_eq!(parse("e"), Ok(Variable::Extension));
    }

    #[test]
    fn parse_extension_with_dot() {
        assert_eq!(parse("E"), Ok(Variable::ExtensionWithDot));
    }

    #[test]
    fn parse_parent() {
        assert_eq!(parse("d"), Ok(Variable::Parent));
    }

    #[test]
    fn parse_parent_file_name() {
        assert_eq!(parse("D"), Ok(Variable::ParentFileName));
    }

    #[test]
    fn parse_local_counter() {
        assert_eq!(parse("c"), Ok(Variable::LocalCounter));
    }

    #[test]
    fn parse_global_counter() {
        assert_eq!(parse("C"), Ok(Variable::GlobalCounter));
    }

    #[test]
    fn parse_regex_capture() {
        assert_eq!(parse("1"), Ok(Variable::RegexCapture(1)));
        assert_eq!(parse("2"), Ok(Variable::RegexCapture(2)));
        assert_eq!(parse("3"), Ok(Variable::RegexCapture(3)));
        assert_eq!(parse("4"), Ok(Variable::RegexCapture(4)));
        assert_eq!(parse("5"), Ok(Variable::RegexCapture(5)));
        assert_eq!(parse("6"), Ok(Variable::RegexCapture(6)));
        assert_eq!(parse("7"), Ok(Variable::RegexCapture(7)));
        assert_eq!(parse("8"), Ok(Variable::RegexCapture(8)));
        assert_eq!(parse("9"), Ok(Variable::RegexCapture(9)));
        assert_eq!(parse("10"), Ok(Variable::RegexCapture(10)));
    }

    #[test]
    fn parse_regex_capture_zero_error() {
        assert_eq!(
            parse("0"),
            Err(parse::Error {
                kind: parse::ErrorKind::RegexCaptureZero,
                range: 0..1,
            }),
        );
    }

    #[test]
    fn parse_uuid() {
        assert_eq!(parse("u"), Ok(Variable::Uuid));
    }

    #[test]
    fn parse_ignore_remaining_chars_after_variable() {
        let mut reader = Reader::from("f_");
        Variable::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_ignore_remaining_chars_after_capture_group() {
        let mut reader = Reader::from("123_");
        Variable::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn parse_empty_error() {
        assert_eq!(
            parse(""),
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedVariable,
                range: 0..0,
            }),
        )
    }

    #[test]
    fn parse_unknown_variable_error() {
        assert_eq!(
            parse("-_"),
            Err(parse::Error {
                kind: parse::ErrorKind::UnknownVariable(Char::Raw('-')),
                range: 0..1,
            }),
        );
    }

    fn parse(string: &str) -> parse::Result<Variable> {
        Variable::parse(&mut Reader::from(string))
    }

    #[test]
    fn eval_path() {
        assert_eq!(
            Variable::Path.eval(&make_context()),
            Ok(String::from("root/parent/file.ext"))
        );
    }

    #[test]
    fn eval_absolute_path() {
        assert_eq!(
            Variable::AbsolutePath.eval(&make_context()),
            Ok(String::from("current_dir/root/parent/file.ext"))
        );
    }

    #[test]
    fn eval_canonical_path() {
        let current_dir = std::env::current_dir().unwrap();
        let file_name = Path::new("Cargo.toml");

        let mut context = make_context();
        context.current_dir = &current_dir;
        context.path = Path::new("Cargo.toml");

        assert_eq!(
            Variable::CanonicalPath.eval(&context),
            Ok(current_dir.join(file_name).to_string_lossy().to_string())
        );
    }

    #[test]
    fn eval_canonical_path_error() {
        assert_eq!(
            Variable::CanonicalPath.eval(&make_context()),
            Err(eval::ErrorKind::CanonicalizationFailed(AnyString(
                String::from("This string is not compared by assertion")
            )))
        );
    }

    #[test]
    fn eval_file_name() {
        assert_eq!(
            Variable::FileName.eval(&make_context()),
            Ok(String::from("file.ext"))
        );
    }

    #[test]
    fn eval_base_name() {
        assert_eq!(
            Variable::BaseName.eval(&make_context()),
            Ok(String::from("file"))
        );
    }

    #[test]
    fn eval_extension() {
        assert_eq!(
            Variable::Extension.eval(&make_context()),
            Ok(String::from("ext"))
        );
    }

    #[test]
    fn eval_extension_missing() {
        let mut context = make_context();
        context.path = Path::new("root/parent/file");
        assert_eq!(Variable::Extension.eval(&context), Ok(String::from("")));
    }

    #[test]
    fn eval_extension_with_dot() {
        assert_eq!(
            Variable::ExtensionWithDot.eval(&make_context()),
            Ok(String::from(".ext"))
        );
    }

    #[test]
    fn eval_extension_with_dot_missing() {
        let mut context = make_context();
        context.path = Path::new("root/parent/file");
        assert_eq!(
            Variable::ExtensionWithDot.eval(&context),
            Ok(String::from(""))
        );
    }

    #[test]
    fn eval_parent() {
        assert_eq!(
            Variable::Parent.eval(&make_context()),
            Ok(String::from("root/parent"))
        );
    }

    #[test]
    fn eval_parent_missing() {
        let mut context = make_context();
        context.path = Path::new("file.ext");
        assert_eq!(Variable::Parent.eval(&context), Ok(String::new()));
    }

    #[test]
    fn eval_parent_file_name() {
        assert_eq!(
            Variable::ParentFileName.eval(&make_context()),
            Ok(String::from("parent"))
        );
    }

    #[test]
    fn eval_parent_file_name_missing() {
        let mut context = make_context();
        context.path = Path::new("file.ext");
        assert_eq!(Variable::ParentFileName.eval(&context), Ok(String::new()));
    }

    #[test]
    fn eval_local_counter() {
        assert_eq!(
            Variable::LocalCounter.eval(&make_context()),
            Ok(String::from("1"))
        );
    }

    #[test]
    fn eval_global_counter() {
        assert_eq!(
            Variable::GlobalCounter.eval(&make_context()),
            Ok(String::from("2"))
        );
    }

    #[test]
    fn eval_regex_capture() {
        assert_eq!(
            Variable::RegexCapture(1).eval(&make_context()),
            Ok(String::from("abc"))
        );
    }

    #[test]
    fn eval_regex_capture_overflow() {
        assert_eq!(
            Variable::RegexCapture(2).eval(&make_context()),
            Ok(String::new())
        );
    }

    #[test]
    fn eval_uuid() {
        let uuid = Variable::Uuid.eval(&make_context()).unwrap();
        let uuid_regex =
            Regex::new("^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
                .unwrap();
        assert!(uuid_regex.is_match(&uuid), format!("{} is UUID v4", uuid));
    }

    #[test]
    fn eval_input_not_utf8() {
        let mut context = make_context();
        context.path = Path::new(make_non_utf8_os_str());
        assert_eq!(
            Variable::Path.eval(&context),
            Err(eval::ErrorKind::InputNotUtf8)
        );
    }

    #[test]
    fn eval_canonicalization_failed() {
        assert_eq!(
            Variable::CanonicalPath.eval(&make_context()),
            Err(eval::ErrorKind::CanonicalizationFailed(AnyString(
                String::from("This string is not compared by assertion")
            )))
        );
    }

    #[test]
    fn fmt() {
        assert_eq!(Variable::Path.to_string(), "Path");
        assert_eq!(Variable::AbsolutePath.to_string(), "Absolute path");
        assert_eq!(Variable::CanonicalPath.to_string(), "Canonical path");
        assert_eq!(Variable::FileName.to_string(), "File name");
        assert_eq!(Variable::BaseName.to_string(), "Base name");
        assert_eq!(Variable::Extension.to_string(), "Extension");
        assert_eq!(Variable::ExtensionWithDot.to_string(), "Extension with dot");
        assert_eq!(Variable::Parent.to_string(), "Parent");
        assert_eq!(Variable::ParentFileName.to_string(), "Parent file name");
        assert_eq!(Variable::LocalCounter.to_string(), "Local counter");
        assert_eq!(Variable::GlobalCounter.to_string(), "Global counter");
        assert_eq!(Variable::RegexCapture(1).to_string(), "Regex capture (1)");
        assert_eq!(Variable::Uuid.to_string(), "UUID");
    }

    #[cfg(any(unix))]
    fn make_non_utf8_os_str<'a>() -> &'a OsStr {
        use std::os::unix::ffi::OsStrExt;
        OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f][..])
    }

    #[cfg(windows)]
    fn make_non_utf8_os_str<'a>() -> &'a OsStr {
        use std::ffi::OsString;
        use std::os::windows::prelude::*;
        OsString::from_wide(&[0x0066, 0x006f, 0xD800, 0x006f][..]).as_os_str()
    }

    fn make_context<'a>() -> eval::Context<'a> {
        eval::Context {
            path: Path::new("root/parent/file.ext"),
            current_dir: Path::new("current_dir"),
            local_counter: 1,
            global_counter: 2,
            regex_captures: Regex::new("(.*)").unwrap().captures("abc"),
        }
    }
}
