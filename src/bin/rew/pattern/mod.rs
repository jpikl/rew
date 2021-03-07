use crate::pattern::filter::Filter;
use crate::pattern::parse::Parsed;
use crate::pattern::parser::Item;
use crate::pattern::parser::Parser;

mod char;
mod column;
mod escape;
pub mod eval;
mod explain;
pub mod filter;
pub mod help;
mod index;
mod integer;
mod lexer;
mod number;
mod padding;
pub mod parse;
mod parser;
pub mod path;
mod range;
mod reader;
pub mod regex;
mod repetition;
mod substitution;
mod switch;
pub mod symbols;
#[cfg(test)]
mod testing;
mod uuid;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    source: String,
    items: Vec<Parsed<Item>>,
}

impl Pattern {
    pub fn parse(source: &str, config: &parse::Config) -> parse::Result<Self> {
        Ok(Self {
            source: String::from(source),
            items: Parser::new(source, config).parse_items()?,
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

                    if let Some(quotes) = context.expression_quotes {
                        output.push(quotes);
                        output.push_str(&value);
                        output.push(quotes);
                    } else {
                        output.push_str(&value);
                    }
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::filter::Filter;
    use super::parse::Parsed;
    use super::parser::Item;
    use super::substitution::Substitution;
    use super::Pattern;
    use crate::pattern::index::Index;
    use crate::pattern::range::Range;
    use crate::utils::AnyString;
    use ntest::*;

    mod parse {
        use super::super::parse::{Config, Error, ErrorKind, Parsed};
        use super::*;

        #[test]
        fn invalid() {
            assert_eq!(
                Pattern::parse("{", &Config::fixture()),
                Err(Error {
                    kind: ErrorKind::UnmatchedExprStart,
                    range: 0..1
                })
            )
        }

        #[test]
        fn valid() {
            assert_eq!(
                Pattern::parse("_%{{f|v}%}_", &Config::fixture()),
                Ok(Pattern {
                    source: String::from("_%{{f|v}%}_"),
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
    }

    mod uses {
        use super::*;

        #[test]
        fn none() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![
                    Parsed::from(Item::Constant(String::from("a"))),
                    Parsed::from(Item::Expression(vec![Parsed::from(Filter::FileName)])),
                ],
            };
            assert_false!(pattern.uses_local_counter());
            assert_false!(pattern.uses_global_counter());
            assert_false!(pattern.uses_regex_capture());
        }

        #[test]
        fn local_counter() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![Parsed::from(Item::Expression(vec![Parsed::from(
                    Filter::LocalCounter,
                )]))],
            };
            assert_true!(pattern.uses_local_counter());
        }

        #[test]
        fn global_counter() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![Parsed::from(Item::Expression(vec![Parsed::from(
                    Filter::GlobalCounter,
                )]))],
            };
            assert_true!(pattern.uses_global_counter());
        }

        #[test]
        fn regex_capture() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![Parsed::from(Item::Expression(vec![Parsed::from(
                    Filter::RegexCapture(1),
                )]))],
            };
            assert_true!(pattern.uses_regex_capture());
        }
    }

    mod eval {
        use super::super::eval::{Context, Error, ErrorKind};
        use super::*;

        #[test]
        fn constant() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![Parsed::from(Item::Constant(String::from("abc")))],
            };
            assert_eq!(
                pattern.eval("", &Context::fixture()),
                Ok(String::from("abc"))
            );
        }

        #[test]
        fn empty_expression() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![Parsed::from(Item::Expression(vec![]))],
            };
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Ok(String::from("dir/file.ext"))
            );
        }

        #[test]
        fn single_filter_expression() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![Parsed::from(Item::Expression(vec![Parsed::from(
                    Filter::FileName,
                )]))],
            };
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Ok(String::from("file.ext"))
            );
        }

        #[test]
        fn multi_filter_expression() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![Parsed::from(Item::Expression(vec![
                    Parsed::from(Filter::FileName),
                    Parsed::from(Filter::ToUppercase),
                ]))],
            };
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Ok(String::from("FILE.EXT"))
            );
        }

        #[test]
        fn multi_constant_and_filter_expressions() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![
                    Parsed::from(Item::Constant(String::from("prefix_"))),
                    Parsed::from(Item::Expression(vec![
                        Parsed::from(Filter::BaseName),
                        Parsed::from(Filter::Substring(Range::<Index>(0, Some(3)))),
                    ])),
                    Parsed::from(Item::Constant(String::from("_"))),
                    Parsed::from(Item::Expression(vec![Parsed::from(Filter::LocalCounter)])),
                    Parsed::from(Item::Constant(String::from("_"))),
                    Parsed::from(Item::Expression(vec![Parsed::from(Filter::GlobalCounter)])),
                    Parsed::from(Item::Constant(String::from("."))),
                    Parsed::from(Item::Expression(vec![
                        Parsed::from(Filter::Extension),
                        Parsed::from(Filter::ToUppercase),
                        Parsed::from(Filter::ReplaceAll(Substitution {
                            target: String::from("X"),
                            replacement: String::new(),
                        })),
                    ])),
                ],
            };
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Ok(String::from("prefix_fil_1_2.ET"))
            );
        }

        #[test]
        fn quotes() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![
                    Parsed::from(Item::Constant(String::from(" "))),
                    Parsed::from(Item::Expression(Vec::new())),
                    Parsed::from(Item::Constant(String::from(" "))),
                ],
            };
            let mut context = Context::fixture();
            context.expression_quotes = Some('\'');
            assert_eq!(
                pattern.eval("dir/file.ext", &context),
                Ok(String::from(" 'dir/file.ext' "))
            );
        }

        #[test]
        fn failure() {
            let pattern = Pattern {
                source: String::new(),
                items: vec![Parsed::from(Item::Expression(vec![Parsed {
                    value: Filter::CanonicalPath,
                    range: 1..2,
                }]))],
            };
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Err(Error {
                    kind: ErrorKind::CanonicalizationFailed(AnyString::any()),
                    value: String::from("dir/file.ext"),
                    cause: &Filter::CanonicalPath,
                    range: &(1..2usize),
                })
            );
        }
    }
}
