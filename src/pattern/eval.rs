use crate::pattern::parse::Parsed;
use crate::pattern::variable::Variable;
use crate::pattern::{Pattern, PatternItem};
use std::path::Path;

#[derive(Debug, PartialEq, Clone)]
pub struct EvalContext<'a> {
    pub path: &'a Path,
    pub local_counter: u32,
    pub global_counter: u32,
    pub capture_groups: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct EvalError<'a> {
    pub message: &'static str,
    pub variable: &'a Parsed<Variable>,
}

impl Pattern {
    pub fn eval(&self, context: &mut EvalContext) -> Result<String, EvalError> {
        let mut output = String::new();

        for item in self.items.iter() {
            match &item.value {
                PatternItem::Constant(string) => output.push_str(string),
                PatternItem::Expression {
                    variable,
                    transforms,
                } => {
                    match variable.value.eval(context) {
                        Ok(mut string) => {
                            for transform in transforms.iter() {
                                string = transform.value.apply(string);
                            }
                            output.push_str(&string)
                        }
                        Err(message) => {
                            return Err(EvalError { message, variable });
                        }
                    };
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::parse::Parsed;
    use crate::pattern::variable::Variable;

    #[test]
    fn constant() {
        let mut context = make_context();
        let pattern = Pattern::parse("abc").unwrap();
        pattern.assert_eval(&mut context, "abc");
    }

    #[test]
    fn expression() {
        let mut context = make_context();
        let pattern = Pattern::parse("{f}").unwrap();
        pattern.assert_eval(&mut context, "ábčd.JPEG");
    }

    #[test]
    fn complex_input() {
        let mut context = make_context();
        let pattern = Pattern::parse("prefix_{b|a}.{e|u|r'e}").unwrap();
        pattern.assert_eval(&mut context, "prefix_abcd.jpg");
    }

    #[test]
    fn multiple_times() {
        let mut context = make_context();
        let pattern = Pattern::parse("image_{1}_{C|>00}_{c}{E}").unwrap();
        pattern.assert_eval(&mut context, "image_abc_02_1.JPEG");
        pattern.assert_eval(&mut context, "image_abc_03_2.JPEG");
        pattern.assert_eval(&mut context, "image_abc_04_3.JPEG");
        assert_eq!(
            context,
            EvalContext {
                path: Path::new("root/parent/ábčd.JPEG"),
                local_counter: 4,
                global_counter: 5,
                capture_groups: vec!["abc".to_string()],
            }
        )
    }

    #[test]
    fn variable_error() {
        let mut context = make_context();
        let pattern = Pattern::parse("image_{2}.{e}").unwrap();
        pattern.assert_eval_error(
            &mut context,
            EvalError {
                message: "Value exceeded number of regex capture groups",
                variable: &Parsed {
                    value: Variable::CaptureGroup(2),
                    start: 7,
                    end: 8,
                },
            },
        )
    }

    fn make_context<'a>() -> EvalContext<'a> {
        EvalContext {
            path: Path::new("root/parent/ábčd.JPEG"),
            local_counter: 1,
            global_counter: 2,
            capture_groups: vec!["abc".to_string()],
        }
    }

    impl Pattern {
        fn assert_eval(&self, context: &mut EvalContext, result: &str) {
            assert_eq!(self.eval(context), Ok(result.to_string()));
        }

        fn assert_eval_error(&self, context: &mut EvalContext, error: EvalError) {
            assert_eq!(self.eval(context), Err(error));
        }
    }
}
