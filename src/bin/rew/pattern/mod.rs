use crate::pattern::filter::Filter;
use crate::pattern::lexer::Lexer;
use crate::pattern::parse::Parsed;
use crate::pattern::parser::Item;
use crate::pattern::parser::Parser;

mod char;
pub mod eval;
mod explain;
mod filter;
pub mod help;
mod lexer;
mod number;
mod padding;
pub mod parse;
mod parser;
mod range;
mod reader;
mod regex;
mod repetition;
mod substitution;
mod symbols;
#[cfg(test)]
mod testing;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    source: String,
    items: Vec<Parsed<Item>>,
}

impl Pattern {
    pub fn parse(source: &str, escape: Option<char>) -> parse::Result<Self> {
        let mut lexer = Lexer::new(source);

        if let Some(escape) = escape {
            lexer.set_escape(escape);
        }

        let mut parser = Parser::new(lexer);
        let items = parser.parse_items()?;

        Ok(Self {
            source: String::from(source),
            items,
        })
    }

    pub fn uses_local_counter(&self) -> bool {
        self.uses_filter(|filter| *filter == Filter::LocalCounter)
    }

    pub fn uses_global_counter(&self) -> bool {
        self.uses_filter(|filter| *filter == Filter::GlobalCounter)
    }

    pub fn uses_regex_capture(&self) -> bool {
        self.uses_filter(|variable| matches!(variable, Filter::RegexCapture(_)))
    }

    fn uses_filter<F: Fn(&Filter) -> bool>(&self, test: F) -> bool {
        self.items.iter().any(|item| {
            if let Item::Expression(filters) = &item.value {
                filters.iter().any(|filter| test(&filter.value))
            } else {
                false
            }
        })
    }

    pub fn eval(&self, input: &str, context: &eval::Context) -> eval::Result<String> {
        let mut output = String::new();

        for item in &self.items {
            match &item.value {
                Item::Constant(value) => output.push_str(value),
                Item::Expression(filters) => {
                    let mut value = input.to_string();

                    for filter in filters.iter() {
                        match filter.value.eval(value, context) {
                            Ok(result) => value = result,
                            Err(kind) => {
                                return Err(eval::Error {
                                    kind,
                                    value: input.to_string(),
                                    cause: &filter.value,
                                    range: &filter.range,
                                });
                            }
                        }
                    }

                    output.push_str(&value);
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
    use crate::pattern::range::IndexRange;
    use crate::pattern::substitution::Substitution;
    use crate::pattern::testing::{make_eval_context, make_parsed};
    use crate::utils::AnyString;
    use ntest::{assert_false, assert_true};

    #[test]
    fn parse_default_escape() {
        assert_eq!(
            Pattern::parse("_#{{f|l}#}_", None),
            Ok(Pattern {
                source: String::from("_#{{f|l}#}_"),
                items: vec![
                    Parsed {
                        value: Item::Constant(String::from("_{")),
                        range: 0..3,
                    },
                    Parsed {
                        value: Item::Expression(vec![
                            Parsed {
                                value: Filter::FileName,
                                range: 4..5,
                            },
                            Parsed {
                                value: Filter::ToLowercase,
                                range: 6..7,
                            }
                        ]),
                        range: 3..8,
                    },
                    Parsed {
                        value: Item::Constant(String::from("}_")),
                        range: 8..11,
                    },
                ]
            })
        )
    }

    #[test]
    fn parse_custom_escape() {
        assert_eq!(
            Pattern::parse("_\\{{f|l}\\}_", Some('\\')),
            Ok(Pattern {
                source: String::from("_\\{{f|l}\\}_"),
                items: vec![
                    Parsed {
                        value: Item::Constant(String::from("_{")),
                        range: 0..3,
                    },
                    Parsed {
                        value: Item::Expression(vec![
                            Parsed {
                                value: Filter::FileName,
                                range: 4..5,
                            },
                            Parsed {
                                value: Filter::ToLowercase,
                                range: 6..7,
                            }
                        ]),
                        range: 3..8,
                    },
                    Parsed {
                        value: Item::Constant(String::from("}_")),
                        range: 8..11,
                    },
                ]
            })
        )
    }

    #[test]
    fn parse_error() {
        assert_eq!(
            Pattern::parse("{", None),
            Err(parse::Error {
                kind: parse::ErrorKind::UnmatchedExprStart,
                range: 0..1
            })
        )
    }

    #[test]
    fn uses_none() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![
                make_parsed(Item::Constant(String::from("a"))),
                make_parsed(Item::Expression(vec![make_parsed(Filter::FileName)])),
            ],
        };
        assert_false!(pattern.uses_local_counter());
        assert_false!(pattern.uses_global_counter());
        assert_false!(pattern.uses_regex_capture());
    }

    #[test]
    fn uses_local_counter() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression(vec![make_parsed(
                Filter::LocalCounter,
            )]))],
        };
        assert_true!(pattern.uses_local_counter());
    }

    #[test]
    fn uses_global_counter() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression(vec![make_parsed(
                Filter::GlobalCounter,
            )]))],
        };
        assert_true!(pattern.uses_global_counter());
    }

    #[test]
    fn uses_regex_capture() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression(vec![make_parsed(
                Filter::RegexCapture(1),
            )]))],
        };
        assert_true!(pattern.uses_regex_capture());
    }

    #[test]
    fn eval_constant() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Constant(String::from("abc")))],
        };
        assert_eq!(
            pattern.eval("", &make_eval_context()),
            Ok(String::from("abc"))
        );
    }

    #[test]
    fn eval_empty_expression() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression(vec![]))],
        };
        assert_eq!(
            pattern.eval("dir/file.ext", &make_eval_context()),
            Ok(String::from("dir/file.ext"))
        );
    }

    #[test]
    fn eval_single_filter_expression() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression(vec![make_parsed(
                Filter::FileName,
            )]))],
        };
        assert_eq!(
            pattern.eval("dir/file.ext", &make_eval_context()),
            Ok(String::from("file.ext"))
        );
    }

    #[test]
    fn eval_multi_filter_expression() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression(vec![
                make_parsed(Filter::FileName),
                make_parsed(Filter::ToUppercase),
            ]))],
        };
        assert_eq!(
            pattern.eval("dir/file.ext", &make_eval_context()),
            Ok(String::from("FILE.EXT"))
        );
    }

    #[test]
    fn eval_multi_constant_and_filter_expressions() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![
                make_parsed(Item::Constant(String::from("prefix_"))),
                make_parsed(Item::Expression(vec![
                    make_parsed(Filter::BaseName),
                    make_parsed(Filter::Substring(IndexRange::new(0, Some(2)))),
                ])),
                make_parsed(Item::Constant(String::from("_"))),
                make_parsed(Item::Expression(vec![make_parsed(Filter::LocalCounter)])),
                make_parsed(Item::Constant(String::from("_"))),
                make_parsed(Item::Expression(vec![make_parsed(Filter::GlobalCounter)])),
                make_parsed(Item::Constant(String::from("."))),
                make_parsed(Item::Expression(vec![
                    make_parsed(Filter::Extension),
                    make_parsed(Filter::ToUppercase),
                    make_parsed(Filter::ReplaceAll(Substitution {
                        target: String::from("X"),
                        replacement: String::from(""),
                    })),
                ])),
            ],
        };
        assert_eq!(
            pattern.eval("dir/file.ext", &make_eval_context()),
            Ok(String::from("prefix_fil_1_2.ET"))
        );
    }

    #[test]
    fn eval_filter_error() {
        let pattern = Pattern {
            source: String::new(),
            items: vec![make_parsed(Item::Expression(vec![Parsed {
                value: Filter::CanonicalPath,
                range: 1..2,
            }]))],
        };
        assert_eq!(
            pattern.eval("dir/file.ext", &make_eval_context()),
            Err(eval::Error {
                kind: eval::ErrorKind::CanonicalizationFailed(AnyString(String::from(
                    "This string is not compared by assertion"
                ))),
                value: String::from("dir/file.ext"),
                cause: &Filter::CanonicalPath,
                range: &(1..2usize),
            })
        );
    }
}
