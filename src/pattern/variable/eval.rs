use crate::pattern::error::ErrorType;
use crate::pattern::eval::EvalContext;
use crate::pattern::variable::Variable;
use std::ffi::OsStr;
use std::path::Path;
use uuid::Uuid;

impl Variable {
    pub fn eval(&self, context: &mut dyn EvalContext) -> Result<String, ErrorType> {
        match self {
            Variable::Filename => Ok(context
                .path()
                .file_name()
                .map_or_else(String::new, os_str_to_string)),

            Variable::Basename => Ok(context
                .path()
                .file_stem()
                .map_or_else(String::new, os_str_to_string)),

            Variable::Extension => Ok(context
                .path()
                .extension()
                .map_or_else(String::new, os_str_to_string)),

            Variable::ExtensionWithDot => {
                Ok(context.path().extension().map_or_else(String::new, |s| {
                    let mut string = os_str_to_string(s);
                    string.insert(0, '.');
                    string
                }))
            }

            Variable::FullDirname => Ok(context
                .path()
                .parent()
                .map(Path::as_os_str)
                .map_or_else(String::new, os_str_to_string)),

            Variable::ParentDirname => Ok(context
                .path()
                .parent()
                .and_then(Path::file_name)
                .map_or_else(String::new, os_str_to_string)),

            Variable::FullPath => Ok(os_str_to_string(context.path().as_os_str())),
            Variable::LocalCounter => Ok(context.local_counter().to_string()),
            Variable::GlobalCounter => Ok(context.global_counter().to_string()),

            Variable::CaptureGroup(index) => Ok(context
                .capture_group(*index)
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
pub mod tests {
    use super::*;
    use crate::pattern::eval::tests::TestEvalContext;
    use regex::Regex;
    use std::path::Path;

    #[test]
    fn filename() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        assert_eq!(
            Variable::Filename.eval(&mut context),
            Ok("file.ext".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn basename() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        assert_eq!(
            Variable::Basename.eval(&mut context),
            Ok("file".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn extension() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        assert_eq!(
            Variable::Extension.eval(&mut context),
            Ok("ext".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn extension_no_ext() {
        let mut context = TestEvalContext::new();
        context.path = Path::new("root/parent/file");
        let final_context = context.clone();
        assert_eq!(Variable::Extension.eval(&mut context), Ok("".to_string()));
        assert_eq!(context, final_context);
    }

    #[test]
    fn extension_with_dot() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        assert_eq!(
            Variable::ExtensionWithDot.eval(&mut context),
            Ok(".ext".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn extension_with_dot_no_ext() {
        let mut context = TestEvalContext::new();
        context.path = Path::new("root/parent/file");
        let final_context = context.clone();
        assert_eq!(
            Variable::ExtensionWithDot.eval(&mut context),
            Ok("".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn full_dirname() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        assert_eq!(
            Variable::FullDirname.eval(&mut context),
            Ok("root/parent".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn full_dirname_no_parent() {
        let mut context = TestEvalContext::new();
        context.path = Path::new("file.ext");
        let final_context = context.clone();
        assert_eq!(Variable::FullDirname.eval(&mut context), Ok(String::new()));
        assert_eq!(context, final_context);
    }

    #[test]
    fn parent_dirname() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        assert_eq!(
            Variable::ParentDirname.eval(&mut context),
            Ok("parent".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn parent_dirname_no_parent() {
        let mut context = TestEvalContext::new();
        context.path = Path::new("file.ext");
        let final_context = context.clone();
        assert_eq!(
            Variable::ParentDirname.eval(&mut context),
            Ok(String::new())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn full_path() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        assert_eq!(
            Variable::FullPath.eval(&mut context),
            Ok("root/parent/file.ext".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn local_counter() {
        let mut context = TestEvalContext::new();
        let mut final_context = context.clone();
        final_context.local_counter = 1;
        assert_eq!(
            Variable::LocalCounter.eval(&mut context),
            Ok("1".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn global_counter() {
        let mut context = TestEvalContext::new();
        let mut final_context = context.clone();
        final_context.global_counter = 1;
        assert_eq!(
            Variable::GlobalCounter.eval(&mut context),
            Ok("1".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn capture_group() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        assert_eq!(
            Variable::CaptureGroup(0).eval(&mut context),
            Ok("abc".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn capture_group_overflow() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        assert_eq!(
            Variable::CaptureGroup(1).eval(&mut context),
            Ok(String::new())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn uuid() {
        let mut context = TestEvalContext::new();
        let final_context = context.clone();
        let uuid = Variable::Uuid.eval(&mut context).unwrap();
        let uuid_regex =
            Regex::new("^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
                .unwrap();
        assert!(uuid_regex.is_match(&uuid), format!("{} is UUID v4", uuid));
        assert_eq!(context, final_context);
    }
}
