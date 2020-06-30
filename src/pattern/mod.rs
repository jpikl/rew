pub use crate::pattern::eval::EvalContext;
use crate::pattern::eval::{EvalError, EvalResult};
pub use crate::pattern::lexer::Lexer;
use crate::pattern::parse::Parsed;
pub use crate::pattern::parser::Parser;
use crate::pattern::parser::PatternItem;
use crate::pattern::variable::Variable;

mod char;
mod eval;
mod filter;
mod lexer;
mod number;
mod parse;
mod parser;
mod range;
mod reader;
mod render;
mod substitution;
mod symbols;
mod variable;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    items: Vec<Parsed<PatternItem>>,
}

impl Pattern {
    pub fn new(items: Vec<Parsed<PatternItem>>) -> Self {
        Self { items }
    }

    pub fn uses_local_counter(&self) -> bool {
        self.uses_variable(|variable| *variable == Variable::LocalCounter)
    }

    pub fn uses_global_counter(&self) -> bool {
        self.uses_variable(|variable| *variable == Variable::GlobalCounter)
    }

    pub fn uses_regex_captures(&self) -> bool {
        self.uses_variable(|variable| matches!(variable, Variable::RegexCapture(_)))
    }

    fn uses_variable<F: Fn(&Variable) -> bool>(&self, test: F) -> bool {
        self.items.iter().any(|item| {
            if let PatternItem::Expression { variable, .. } = &item.value {
                test(&variable.value)
            } else {
                false
            }
        })
    }

    pub fn eval(&self, context: &EvalContext) -> EvalResult<String> {
        let mut output = String::new();

        for item in self.items.iter() {
            match &item.value {
                PatternItem::Constant(string) => output.push_str(string),
                PatternItem::Expression { variable, filters } => {
                    match variable.value.eval(context) {
                        Ok(mut string) => {
                            for filter in filters.iter() {
                                string = filter.value.apply(string);
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
    use crate::pattern::filter::Filter;
    use crate::pattern::parse::Parsed;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;
    use regex::Regex;
    use std::path::Path;

    #[test]
    fn uses_none() {
        let items = vec![parsed(PatternItem::Expression {
            variable: parsed(Variable::Filename),
            filters: Vec::new(),
        })];
        let pattern = Pattern::new(items);
        assert_eq!(pattern.uses_local_counter(), false);
        assert_eq!(pattern.uses_global_counter(), false);
        assert_eq!(pattern.uses_regex_captures(), false);
    }

    #[test]
    fn uses_local_counter() {
        let items = vec![parsed(PatternItem::Expression {
            variable: parsed(Variable::LocalCounter),
            filters: Vec::new(),
        })];
        assert_eq!(Pattern::new(items).uses_local_counter(), true);
    }

    #[test]
    fn uses_global_counter() {
        let items = vec![parsed(PatternItem::Expression {
            variable: parsed(Variable::GlobalCounter),
            filters: Vec::new(),
        })];
        assert_eq!(Pattern::new(items).uses_global_counter(), true);
    }

    #[test]
    fn uses_global_regex_captures() {
        let items = vec![parsed(PatternItem::Expression {
            variable: parsed(Variable::RegexCapture(1)),
            filters: Vec::new(),
        })];
        assert_eq!(Pattern::new(items).uses_regex_captures(), true);
    }

    #[test]
    fn constant() {
        let items = vec![parsed(PatternItem::Constant(String::from("abc")))];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok(String::from("abc"))
        );
    }

    #[test]
    fn expression() {
        let items = vec![parsed(PatternItem::Expression {
            variable: parsed(Variable::Filename),
            filters: Vec::new(),
        })];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok(String::from("file.ext"))
        );
    }

    #[test]
    fn expression_single_filter() {
        let items = vec![parsed(PatternItem::Expression {
            variable: parsed(Variable::Filename),
            filters: vec![parsed(Filter::Uppercase)],
        })];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok(String::from("FILE.EXT"))
        );
    }

    #[test]
    fn expression_multiple_filters() {
        let items = vec![parsed(PatternItem::Expression {
            variable: parsed(Variable::Filename),
            filters: vec![
                parsed(Filter::Uppercase),
                parsed(Filter::Substring(Range::To(4))),
            ],
        })];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok(String::from("FILE"))
        );
    }

    #[test]
    fn multiple_constants_and_expressions() {
        let items = vec![
            parsed(PatternItem::Constant(String::from("prefix_"))),
            parsed(PatternItem::Expression {
                variable: parsed(Variable::Basename),
                filters: vec![parsed(Filter::Substring(Range::To(3)))],
            }),
            parsed(PatternItem::Constant(String::from("_"))),
            parsed(PatternItem::Expression {
                variable: parsed(Variable::RegexCapture(1)),
                filters: Vec::new(),
            }),
            parsed(PatternItem::Constant(String::from("_"))),
            parsed(PatternItem::Expression {
                variable: parsed(Variable::LocalCounter),
                filters: Vec::new(),
            }),
            parsed(PatternItem::Constant(String::from("_"))),
            parsed(PatternItem::Expression {
                variable: parsed(Variable::GlobalCounter),
                filters: Vec::new(),
            }),
            parsed(PatternItem::Constant(String::from("."))),
            parsed(PatternItem::Expression {
                variable: parsed(Variable::Extension),
                filters: vec![
                    parsed(Filter::Uppercase),
                    parsed(Filter::ReplaceAll(Substitution {
                        value: String::from("X"),
                        replacement: String::from(""),
                    })),
                ],
            }),
        ];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok(String::from("prefix_fil_abc_1_2.ET"))
        );
    }

    fn parsed<T>(value: T) -> Parsed<T> {
        Parsed { value, range: 0..0 }
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
