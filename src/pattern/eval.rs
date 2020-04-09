use crate::pattern::error::ErrorType;
use crate::pattern::parse::Parsed;
use crate::pattern::parser::PatternItem;
use crate::pattern::variable::Variable;
use crate::pattern::Pattern;
use std::path::Path;

pub trait EvalContext {
    fn path(&self) -> &Path;
    fn local_counter(&mut self) -> u32;
    fn global_counter(&mut self) -> u32;
    fn capture_group(&self, index: usize) -> Option<&str>;
}

#[derive(Debug, PartialEq)]
pub struct EvalError<'a> {
    pub typ: ErrorType,
    pub variable: &'a Parsed<Variable>,
}

impl Pattern {
    pub fn eval<'a>(&self, context: &'a mut dyn EvalContext) -> Result<String, EvalError> {
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
                        Err(typ) => {
                            return Err(EvalError { typ, variable });
                        }
                    };
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn constant() {
        let mut context = TestEvalContext::new();
        let pattern = Pattern::parse("abc").unwrap();
        pattern.assert_eval(&mut context, "abc");
    }

    #[test]
    fn expression() {
        let mut context = TestEvalContext::new();
        let pattern = Pattern::parse("{f}").unwrap();
        pattern.assert_eval(&mut context, "file.ext");
    }

    #[test]
    fn complex_input() {
        let mut context = TestEvalContext::new();
        let pattern = Pattern::parse("prefix_{b|n1-3}.{e|u|r'X}").unwrap();
        pattern.assert_eval(&mut context, "prefix_fil.ET");
    }

    #[test]
    fn multiple_times() {
        let mut context = TestEvalContext::new();
        context.global_counter = 1;
        let pattern = Pattern::parse("prefix_{1}_{C|<00}_{c}{E}").unwrap();
        pattern.assert_eval(&mut context, "prefix_abc_02_1.ext");
        pattern.assert_eval(&mut context, "prefix_abc_03_2.ext");
        pattern.assert_eval(&mut context, "prefix_abc_04_3.ext");
        assert_eq!(
            context,
            TestEvalContext {
                path: Path::new("root/parent/file.ext"),
                local_counter: 3,
                global_counter: 4,
                capture_groups: vec!["abc".to_string()],
            }
        )
    }

    impl Pattern {
        fn assert_eval(&self, context: &mut dyn EvalContext, result: &str) {
            assert_eq!(self.eval(context), Ok(result.to_string()));
        }

        fn assert_eval_error(&self, context: &mut dyn EvalContext, error: EvalError) {
            assert_eq!(self.eval(context), Err(error));
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct TestEvalContext<'a> {
        pub path: &'a Path,
        pub local_counter: u32,
        pub global_counter: u32,
        pub capture_groups: Vec<String>,
    }

    impl<'a> TestEvalContext<'a> {
        pub fn new() -> Self {
            Self {
                path: Path::new("root/parent/file.ext"),
                local_counter: 0,
                global_counter: 0,
                capture_groups: vec!["abc".to_string()],
            }
        }
    }

    impl<'a> EvalContext for TestEvalContext<'a> {
        fn path(&self) -> &Path {
            self.path
        }

        fn local_counter(&mut self) -> u32 {
            self.local_counter += 1;
            self.local_counter
        }

        fn global_counter(&mut self) -> u32 {
            self.global_counter += 1;
            self.global_counter
        }

        fn capture_group(&self, index: usize) -> Option<&str> {
            if index >= self.capture_groups.len() {
                None
            } else {
                Some(&self.capture_groups[index])
            }
        }
    }
}
