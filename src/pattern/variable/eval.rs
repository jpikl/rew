use crate::pattern::error::EvalErrorKind;
use crate::pattern::eval::EvalContext;
use crate::pattern::variable::Variable;
use std::ffi::OsStr;
use std::path::Path;
use uuid::Uuid;

impl Variable {
    pub fn eval(&self, context: &EvalContext) -> Result<String, EvalErrorKind> {
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

            Variable::FullDirname => Ok(context
                .path
                .parent()
                .map(Path::as_os_str)
                .map_or_else(String::new, os_str_to_string)),

            Variable::ParentDirname => Ok(context
                .path
                .parent()
                .and_then(Path::file_name)
                .map_or_else(String::new, os_str_to_string)),

            Variable::FullPath => Ok(os_str_to_string(context.path.as_os_str())),
            Variable::LocalCounter => Ok(context.local_counter.to_string()),
            Variable::GlobalCounter => Ok(context.global_counter.to_string()),

            Variable::RegexCapture(index) => Ok(context
                .regex_captures
                .as_ref()
                .and_then(|captures| captures.get(*index))
                .map(|r#match| r#match.as_str())
                .map_or_else(String::new, String::from)),

            Variable::Uuid => {
                let mut buffer = Uuid::encode_buffer();
                let str = Uuid::new_v4().to_hyphenated().encode_lower(&mut buffer);
                Ok((*str).to_string())
            }
        }
    }
}

fn os_str_to_string(str: &OsStr) -> String {
    // TODO return error instead of lossy conversion
    str.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use std::path::Path;

    #[test]
    fn filename() {
        assert_eq!(
            Variable::Filename.eval(&make_context()),
            Ok("file.ext".to_string())
        );
    }

    #[test]
    fn basename() {
        assert_eq!(
            Variable::Basename.eval(&make_context()),
            Ok("file".to_string())
        );
    }

    #[test]
    fn extension() {
        assert_eq!(
            Variable::Extension.eval(&make_context()),
            Ok("ext".to_string())
        );
    }

    #[test]
    fn extension_no_ext() {
        let mut context = make_context();
        context.path = Path::new("root/parent/file");
        assert_eq!(Variable::Extension.eval(&context), Ok("".to_string()));
    }

    #[test]
    fn extension_with_dot() {
        assert_eq!(
            Variable::ExtensionWithDot.eval(&make_context()),
            Ok(".ext".to_string())
        );
    }

    #[test]
    fn extension_with_dot_no_ext() {
        let mut context = make_context();
        context.path = Path::new("root/parent/file");
        assert_eq!(
            Variable::ExtensionWithDot.eval(&context),
            Ok("".to_string())
        );
    }

    #[test]
    fn full_dirname() {
        assert_eq!(
            Variable::FullDirname.eval(&make_context()),
            Ok("root/parent".to_string())
        );
    }

    #[test]
    fn full_dirname_no_parent() {
        let mut context = make_context();
        context.path = Path::new("file.ext");
        assert_eq!(Variable::FullDirname.eval(&context), Ok(String::new()));
    }

    #[test]
    fn parent_dirname() {
        assert_eq!(
            Variable::ParentDirname.eval(&make_context()),
            Ok("parent".to_string())
        );
    }

    #[test]
    fn parent_dirname_no_parent() {
        let mut context = make_context();
        context.path = Path::new("file.ext");
        assert_eq!(Variable::ParentDirname.eval(&context), Ok(String::new()));
    }

    #[test]
    fn full_path() {
        assert_eq!(
            Variable::FullPath.eval(&make_context()),
            Ok("root/parent/file.ext".to_string())
        );
    }

    #[test]
    fn local_counter() {
        assert_eq!(
            Variable::LocalCounter.eval(&make_context()),
            Ok("1".to_string())
        );
    }

    #[test]
    fn global_counter() {
        assert_eq!(
            Variable::GlobalCounter.eval(&make_context()),
            Ok("2".to_string())
        );
    }

    #[test]
    fn regex_capture() {
        assert_eq!(
            Variable::RegexCapture(1).eval(&make_context()),
            Ok("abc".to_string())
        );
    }

    #[test]
    fn regex_capture_overflow() {
        assert_eq!(
            Variable::RegexCapture(2).eval(&make_context()),
            Ok(String::new())
        );
    }

    #[test]
    fn uuid() {
        let uuid = Variable::Uuid.eval(&make_context()).unwrap();
        let uuid_regex =
            Regex::new("^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
                .unwrap();
        assert!(uuid_regex.is_match(&uuid), format!("{} is UUID v4", uuid));
    }

    fn make_context<'a>() -> EvalContext<'a> {
        EvalContext {
            path: Path::new("root/parent/file.ext"),
            local_counter: 1,
            global_counter: 2,
            regex_captures: Regex::new("(.*)").unwrap().captures("abc"),
        }
    }
}
