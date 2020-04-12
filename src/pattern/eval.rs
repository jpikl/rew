use crate::pattern::error::ErrorType;
use crate::pattern::parse::Parsed;
use crate::pattern::parser::PatternItem;
use crate::pattern::variable::Variable;
use crate::pattern::Pattern;
use std::path::Path;

pub struct EvalContext<'a> {
    pub path: &'a Path,
    pub local_counter: u32,
    pub global_counter: u32,
    pub regex_captures: Option<regex::Captures<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct EvalError<'a> {
    pub typ: ErrorType,
    pub variable: &'a Parsed<Variable>,
}

impl Pattern {
    pub fn eval(&self, context: &EvalContext) -> Result<String, EvalError> {
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
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn constant() {
        assert_eval("abc", "abc");
    }

    #[test]
    fn expression() {
        assert_eval("{f}", "file.ext");
    }

    #[test]
    fn complex_input() {
        assert_eval(
            "prefix_{b|n1-3}_{1}_{c}_{C}.{e|u|r'X}",
            "prefix_fil_abc_1_2.ET",
        );
    }

    fn assert_eval(pattern: &str, result: &str) {
        let pattern = Pattern::parse(pattern).unwrap();
        let context = EvalContext {
            path: Path::new("root/parent/file.ext"),
            local_counter: 1,
            global_counter: 2,
            regex_captures: Regex::new("(.*)").unwrap().captures("abc"),
        };
        assert_eq!(pattern.eval(&context), Ok(String::from(result)));
    }
}
