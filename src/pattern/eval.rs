use crate::pattern::error::{EvalError, EvalResult};
use crate::pattern::parser::PatternItem;
use crate::pattern::Pattern;
use std::path::Path;

pub struct EvalContext<'a> {
    pub path: &'a Path,
    pub local_counter: u32,
    pub global_counter: u32,
    pub regex_captures: Option<regex::Captures<'a>>,
}

impl Pattern {
    pub fn eval(&self, context: &EvalContext) -> EvalResult<String> {
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
                        Err(kind) => {
                            return Err(EvalError { kind, item });
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
    use crate::pattern::lexer::Parsed;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;
    use crate::pattern::transform::Transform;
    use crate::pattern::variable::Variable;
    use regex::Regex;

    #[test]
    fn constant() {
        let items = vec![Parsed::dummy(PatternItem::Constant("abc".to_string()))];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok("abc".to_string())
        );
    }

    #[test]
    fn expression() {
        let items = vec![Parsed::dummy(PatternItem::Expression {
            variable: Parsed::dummy(Variable::Filename),
            transforms: Vec::new(),
        })];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok("file.ext".to_string())
        );
    }

    #[test]
    fn expression_single_transform() {
        let items = vec![Parsed::dummy(PatternItem::Expression {
            variable: Parsed::dummy(Variable::Filename),
            transforms: vec![Parsed::dummy(Transform::Uppercase)],
        })];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok("FILE.EXT".to_string())
        );
    }

    #[test]
    fn expression_multiple_transforms() {
        let items = vec![Parsed::dummy(PatternItem::Expression {
            variable: Parsed::dummy(Variable::Filename),
            transforms: vec![
                Parsed::dummy(Transform::Uppercase),
                Parsed::dummy(Transform::Substring(Range::To(4))),
            ],
        })];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok("FILE".to_string())
        );
    }

    #[test]
    fn multiple_constants_and_expressions() {
        let items = vec![
            Parsed::dummy(PatternItem::Constant("prefix_".to_string())),
            Parsed::dummy(PatternItem::Expression {
                variable: Parsed::dummy(Variable::Basename),
                transforms: vec![Parsed::dummy(Transform::Substring(Range::To(3)))],
            }),
            Parsed::dummy(PatternItem::Constant("_".to_string())),
            Parsed::dummy(PatternItem::Expression {
                variable: Parsed::dummy(Variable::RegexCapture(1)),
                transforms: Vec::new(),
            }),
            Parsed::dummy(PatternItem::Constant("_".to_string())),
            Parsed::dummy(PatternItem::Expression {
                variable: Parsed::dummy(Variable::LocalCounter),
                transforms: Vec::new(),
            }),
            Parsed::dummy(PatternItem::Constant("_".to_string())),
            Parsed::dummy(PatternItem::Expression {
                variable: Parsed::dummy(Variable::GlobalCounter),
                transforms: Vec::new(),
            }),
            Parsed::dummy(PatternItem::Constant(".".to_string())),
            Parsed::dummy(PatternItem::Expression {
                variable: Parsed::dummy(Variable::Extension),
                transforms: vec![
                    Parsed::dummy(Transform::Uppercase),
                    Parsed::dummy(Transform::ReplaceAll(Substitution {
                        value: "X".to_string(),
                        replacement: "".to_string(),
                    })),
                ],
            }),
        ];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok("prefix_fil_abc_1_2.ET".to_string())
        );
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
