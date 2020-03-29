use crate::pattern::error::ParseError;
use crate::pattern::number::parse_usize;
use crate::pattern::reader::Reader;
use std::ffi::OsStr;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum Variable {
    Filename,
    Basename,
    Extension,
    ExtensionWithDot,
    LocalCounter,
    GlobalCounter,
    CaptureGroup(usize),
    Uuid,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EvalContext<'a> {
    pub path: &'a Path,
    pub local_counter: u32,
    pub global_counter: u32,
    pub capture_croups: Vec<String>,
}

impl Variable {
    pub fn parse(string: &str) -> Result<Variable, ParseError> {
        let mut reader = Reader::new(string);

        let variable = match reader.peek() {
            Some('0'..='9') => Variable::CaptureGroup(parse_usize(&mut reader)?),
            Some(ch) => {
                reader.next();
                match ch {
                    'f' => Variable::Filename,
                    'b' => Variable::Basename,
                    'e' => Variable::Extension,
                    'E' => Variable::ExtensionWithDot,
                    'c' => Variable::LocalCounter,
                    'C' => Variable::GlobalCounter,
                    'u' => Variable::Uuid,
                    _ => {
                        return Err(ParseError {
                            message: "Unknown variable",
                            position: 0,
                        });
                    }
                }
            }
            None => {
                return Err(ParseError {
                    message: "Empty input",
                    position: 0,
                })
            }
        };

        if reader.peek().is_none() {
            Ok(variable)
        } else {
            Err(ParseError {
                message: "Unexpected character",
                position: reader.position(),
            })
        }
    }

    pub fn eval(&self, context: &mut EvalContext) -> Result<String, &'static str> {
        match self {
            Variable::Filename => Ok(context
                .path
                .file_name()
                .map_or_else(String::new, os_str_to_string)),
            Variable::Basename => Ok(context
                .path
                .file_stem()
                .map_or_else(String::new, os_str_to_string)),
            Variable::Extension => Ok(context
                .path
                .extension()
                .map_or_else(String::new, os_str_to_string)),
            Variable::ExtensionWithDot => {
                Ok(context.path.extension().map_or_else(String::new, |s| {
                    let mut string = os_str_to_string(s);
                    string.insert(0, '.');
                    string
                }))
            }
            Variable::LocalCounter => {
                let counter = context.local_counter;
                context.local_counter += 1;
                Ok(counter.to_string())
            }
            Variable::GlobalCounter => {
                let counter = context.global_counter;
                context.global_counter += 1;
                Ok(counter.to_string())
            }
            Variable::CaptureGroup(number) => {
                if *number == 0 {
                    Err("Regex capture groups are numbered from 1")
                } else if *number > context.capture_croups.len() {
                    Err("Value exceeded number of regex capture groups")
                } else {
                    Ok(context.capture_croups[*number - 1].clone())
                }
            }
            Variable::Uuid => {
                let mut buffer = Uuid::encode_buffer();
                let str = Uuid::new_v4().to_hyphenated().encode_lower(&mut buffer);
                Ok((*str).to_string())
            }
        }
    }
}

fn os_str_to_string(str: &OsStr) -> String {
    str.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn parse_filename() {
        assert_eq!(Variable::parse("f"), Ok(Variable::Filename));
    }

    #[test]
    fn parse_basename() {
        assert_eq!(Variable::parse("b"), Ok(Variable::Basename));
    }

    #[test]
    fn parse_extension() {
        assert_eq!(Variable::parse("e"), Ok(Variable::Extension));
    }

    #[test]
    fn parse_extension_with_dot() {
        assert_eq!(Variable::parse("E"), Ok(Variable::ExtensionWithDot));
    }

    #[test]
    fn parse_local_counter() {
        assert_eq!(Variable::parse("c"), Ok(Variable::LocalCounter));
    }

    #[test]
    fn parse_global_counter() {
        assert_eq!(Variable::parse("C"), Ok(Variable::GlobalCounter));
    }

    #[test]
    fn parse_regex_group() {
        assert_eq!(Variable::parse("1"), Ok(Variable::CaptureGroup(1)));
        assert_eq!(Variable::parse("2"), Ok(Variable::CaptureGroup(2)));
        assert_eq!(Variable::parse("3"), Ok(Variable::CaptureGroup(3)));
        assert_eq!(Variable::parse("4"), Ok(Variable::CaptureGroup(4)));
        assert_eq!(Variable::parse("5"), Ok(Variable::CaptureGroup(5)));
        assert_eq!(Variable::parse("6"), Ok(Variable::CaptureGroup(6)));
        assert_eq!(Variable::parse("7"), Ok(Variable::CaptureGroup(7)));
        assert_eq!(Variable::parse("8"), Ok(Variable::CaptureGroup(8)));
        assert_eq!(Variable::parse("9"), Ok(Variable::CaptureGroup(9)));
        assert_eq!(Variable::parse("10"), Ok(Variable::CaptureGroup(10)));
    }

    #[test]
    fn parse_uuid() {
        assert_eq!(Variable::parse("u"), Ok(Variable::Uuid));
    }

    #[test]
    fn parse_unknown_variable_as_error() {
        assert_eq!(
            Variable::parse("_"),
            Err(ParseError {
                message: "Unknown variable",
                position: 0,
            })
        );
    }

    #[test]
    fn parse_unexpected_character_as_error() {
        assert_eq!(
            Variable::parse("f_"),
            Err(ParseError {
                message: "Unexpected character",
                position: 1,
            })
        );
        assert_eq!(
            Variable::parse("123_"),
            Err(ParseError {
                message: "Unexpected character",
                position: 3,
            })
        );
    }

    fn make_eval_context<'a>() -> EvalContext<'a> {
        EvalContext {
            path: Path::new("root/parent/image.png"),
            local_counter: 0,
            global_counter: 0,
            capture_croups: vec!["abc".to_string()],
        }
    }

    #[test]
    fn eval_filename() {
        let mut context = make_eval_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::Filename.eval(&mut context),
            Ok("image.png".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_basename() {
        let mut context = make_eval_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::Basename.eval(&mut context),
            Ok("image".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_extension() {
        let mut context = make_eval_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::Extension.eval(&mut context),
            Ok("png".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_extension_no_ext() {
        let mut context = make_eval_context();
        context.path = Path::new("root/parent/image");
        let final_context = context.clone();
        assert_eq!(Variable::Extension.eval(&mut context), Ok("".to_string()));
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_extension_with_dot() {
        let mut context = make_eval_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::ExtensionWithDot.eval(&mut context),
            Ok(".png".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_extension_with_dot_no_ext() {
        let mut context = make_eval_context();
        context.path = Path::new("root/parent/image");
        let final_context = context.clone();
        assert_eq!(
            Variable::ExtensionWithDot.eval(&mut context),
            Ok("".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_local_counter() {
        let mut context = make_eval_context();
        let mut final_context = context.clone();
        final_context.local_counter = 1;
        assert_eq!(
            Variable::LocalCounter.eval(&mut context),
            Ok("0".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_global_counter() {
        let mut context = make_eval_context();
        let mut final_context = context.clone();
        final_context.global_counter = 1;
        assert_eq!(
            Variable::GlobalCounter.eval(&mut context),
            Ok("0".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_capture_group_below_one() {
        let mut context = make_eval_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::CaptureGroup(0).eval(&mut context),
            Err("Regex capture groups are numbered from 1")
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_capture_group_over_max() {
        let mut context = make_eval_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::CaptureGroup(2).eval(&mut context),
            Err("Value exceeded number of regex capture groups")
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn eval_uuid() {
        let mut context = make_eval_context();
        let final_context = context.clone();
        let uuid = Variable::Uuid.eval(&mut context).unwrap();
        let uuid_regex =
            Regex::new("^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
                .unwrap();
        assert!(uuid_regex.is_match(&uuid), format!("{} is UUID v4", uuid));
        assert_eq!(context, final_context);
    }

    #[test]
    fn parse_empty_as_error() {
        assert_eq!(
            Variable::parse(""),
            Err(ParseError {
                message: "Empty input",
                position: 0,
            })
        )
    }
}
