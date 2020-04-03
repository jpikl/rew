use crate::pattern::variable::Variable;
use crate::pattern::EvalContext;
use std::ffi::OsStr;
use std::path::Path;
use uuid::Uuid;

impl Variable {
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
                } else if *number > context.capture_groups.len() {
                    Err("Value exceeded number of regex capture groups")
                } else {
                    Ok(context.capture_groups[*number - 1].clone())
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
    fn filename() {
        let mut context = make_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::Filename.eval(&mut context),
            Ok("image.png".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn basename() {
        let mut context = make_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::Basename.eval(&mut context),
            Ok("image".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn extension() {
        let mut context = make_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::Extension.eval(&mut context),
            Ok("png".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn extension_no_ext() {
        let mut context = make_context();
        context.path = Path::new("root/parent/image");
        let final_context = context.clone();
        assert_eq!(Variable::Extension.eval(&mut context), Ok("".to_string()));
        assert_eq!(context, final_context);
    }

    #[test]
    fn extension_with_dot() {
        let mut context = make_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::ExtensionWithDot.eval(&mut context),
            Ok(".png".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn extension_with_dot_no_ext() {
        let mut context = make_context();
        context.path = Path::new("root/parent/image");
        let final_context = context.clone();
        assert_eq!(
            Variable::ExtensionWithDot.eval(&mut context),
            Ok("".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn local_counter() {
        let mut context = make_context();
        let mut final_context = context.clone();
        final_context.local_counter = 1;
        assert_eq!(
            Variable::LocalCounter.eval(&mut context),
            Ok("0".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn global_counter() {
        let mut context = make_context();
        let mut final_context = context.clone();
        final_context.global_counter = 1;
        assert_eq!(
            Variable::GlobalCounter.eval(&mut context),
            Ok("0".to_string())
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn capture_group_below_one() {
        let mut context = make_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::CaptureGroup(0).eval(&mut context),
            Err("Regex capture groups are numbered from 1")
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn capture_group_over_max() {
        let mut context = make_context();
        let final_context = context.clone();
        assert_eq!(
            Variable::CaptureGroup(2).eval(&mut context),
            Err("Value exceeded number of regex capture groups")
        );
        assert_eq!(context, final_context);
    }

    #[test]
    fn uuid() {
        let mut context = make_context();
        let final_context = context.clone();
        let uuid = Variable::Uuid.eval(&mut context).unwrap();
        let uuid_regex =
            Regex::new("^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
                .unwrap();
        assert!(uuid_regex.is_match(&uuid), format!("{} is UUID v4", uuid));
        assert_eq!(context, final_context);
    }

    fn make_context<'a>() -> EvalContext<'a> {
        EvalContext {
            path: Path::new("root/parent/image.png"),
            local_counter: 0,
            global_counter: 0,
            capture_groups: vec!["abc".to_string()],
        }
    }
}
