use crate::pattern::eval::{Context, Error, ErrorCause, Result};
pub use crate::pattern::lexer::Lexer;
use crate::pattern::parse::Output;
use crate::pattern::parser::Item;
pub use crate::pattern::parser::Parser;
use crate::pattern::variable::Variable;

mod char;
pub mod eval;
mod filter;
mod lexer;
mod number;
pub mod parse;
mod parser;
mod range;
mod reader;
mod render;
mod substitution;
mod symbols;
mod variable;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    items: Vec<Output<Item>>,
}

impl Pattern {
    pub fn new(items: Vec<Output<Item>>) -> Self {
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
            if let Item::Expression { variable, .. } = &item.value {
                test(&variable.value)
            } else {
                false
            }
        })
    }

    pub fn eval(&self, context: &Context) -> Result<String> {
        let mut output = String::new();

        for item in self.items.iter() {
            match &item.value {
                Item::Constant(string) => output.push_str(string),
                Item::Expression { variable, filters } => {
                    match variable.value.eval(context) {
                        Ok(mut string) => {
                            for filter in filters.iter() {
                                match filter.value.eval(string) {
                                    Ok(result) => string = result,
                                    Err(kind) => {
                                        return Err(Error {
                                            kind,
                                            cause: ErrorCause::Filter(filter),
                                        });
                                    }
                                }
                            }
                            output.push_str(&string)
                        }
                        Err(kind) => {
                            return Err(Error {
                                kind,
                                cause: ErrorCause::Variable(variable),
                            });
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
    use crate::pattern::parse::Output;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;
    use regex::Regex;
    use std::path::Path;

    #[test]
    fn uses_none() {
        let items = vec![parsed(Item::Expression {
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
        let items = vec![parsed(Item::Expression {
            variable: parsed(Variable::LocalCounter),
            filters: Vec::new(),
        })];
        assert_eq!(Pattern::new(items).uses_local_counter(), true);
    }

    #[test]
    fn uses_global_counter() {
        let items = vec![parsed(Item::Expression {
            variable: parsed(Variable::GlobalCounter),
            filters: Vec::new(),
        })];
        assert_eq!(Pattern::new(items).uses_global_counter(), true);
    }

    #[test]
    fn uses_global_regex_captures() {
        let items = vec![parsed(Item::Expression {
            variable: parsed(Variable::RegexCapture(1)),
            filters: Vec::new(),
        })];
        assert_eq!(Pattern::new(items).uses_regex_captures(), true);
    }

    #[test]
    fn constant() {
        let items = vec![parsed(Item::Constant(String::from("abc")))];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok(String::from("abc"))
        );
    }

    #[test]
    fn expression() {
        let items = vec![parsed(Item::Expression {
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
        let items = vec![parsed(Item::Expression {
            variable: parsed(Variable::Filename),
            filters: vec![parsed(Filter::ToUppercase)],
        })];
        assert_eq!(
            Pattern::new(items).eval(&make_context()),
            Ok(String::from("FILE.EXT"))
        );
    }

    #[test]
    fn expression_multiple_filters() {
        let items = vec![parsed(Item::Expression {
            variable: parsed(Variable::Filename),
            filters: vec![
                parsed(Filter::ToUppercase),
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
            parsed(Item::Constant(String::from("prefix_"))),
            parsed(Item::Expression {
                variable: parsed(Variable::Basename),
                filters: vec![parsed(Filter::Substring(Range::To(3)))],
            }),
            parsed(Item::Constant(String::from("_"))),
            parsed(Item::Expression {
                variable: parsed(Variable::RegexCapture(1)),
                filters: Vec::new(),
            }),
            parsed(Item::Constant(String::from("_"))),
            parsed(Item::Expression {
                variable: parsed(Variable::LocalCounter),
                filters: Vec::new(),
            }),
            parsed(Item::Constant(String::from("_"))),
            parsed(Item::Expression {
                variable: parsed(Variable::GlobalCounter),
                filters: Vec::new(),
            }),
            parsed(Item::Constant(String::from("."))),
            parsed(Item::Expression {
                variable: parsed(Variable::Extension),
                filters: vec![
                    parsed(Filter::ToUppercase),
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

    fn parsed<T>(value: T) -> Output<T> {
        Output { value, range: 0..0 }
    }

    fn make_context<'a>() -> Context<'a> {
        Context {
            path: Path::new("root/parent/file.ext"),
            local_counter: 1,
            global_counter: 2,
            regex_captures: Regex::new("(.*)").unwrap().captures("abc"),
        }
    }
}
