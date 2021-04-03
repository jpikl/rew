use crate::pattern::filter::Filter;
use crate::pattern::parser::Parser;
use crate::pattern::parser::{Item, ParsedItem};

mod char;
mod escape;
pub mod eval;
mod explain;
mod field;
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
mod uuid;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    source: String,
    items: Vec<ParsedItem>,
}

impl Pattern {
    pub fn parse(source: &str, config: &parse::Config) -> parse::Result<Self> {
        Ok(Self {
            source: source.into(),
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
impl From<Vec<ParsedItem>> for Pattern {
    fn from(items: Vec<ParsedItem>) -> Self {
        Self {
            source: String::new(),
            items,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::filter::Filter;
    use super::parse::Parsed;
    use super::parser::Item;
    use super::Pattern;
    use crate::utils::AnyString;
    use test_case::test_case;

    mod parse {
        use super::super::parse::{Config, Error, ErrorKind, Parsed};
        use super::*;

        #[test]
        fn err() {
            assert_eq!(
                Pattern::parse("{", &Config::fixture()),
                Err(Error {
                    kind: ErrorKind::UnmatchedExprStart,
                    range: 0..1
                })
            )
        }

        #[test]
        fn ok() {
            assert_eq!(
                Pattern::parse("_%{{f|v}%}_", &Config::fixture()),
                Ok(Pattern {
                    source: "_%{{f|v}%}_".into(),
                    items: vec![
                        Parsed {
                            value: Item::Constant("_{".into()),
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
                            value: Item::Constant("}_".into()),
                            range: 8..11,
                        },
                    ]
                })
            )
        }
    }

    #[test_case(Filter::FileName,        false, false, false ; "none")]
    #[test_case(Filter::LocalCounter,    true,  false, false ; "local counter")]
    #[test_case(Filter::GlobalCounter,   false, true,  false ; "global counter")]
    #[test_case(Filter::RegexCapture(1), false, false, true  ; "regex capture")]
    fn uses(filter: Filter, local_counter: bool, global_counter: bool, regex_capture: bool) {
        let pattern = Pattern::from(vec![
            Parsed::from(Item::Constant("a".into())),
            Parsed::from(Item::Expression(vec![Parsed::from(filter)])),
        ]);
        assert_eq!(pattern.uses_local_counter(), local_counter);
        assert_eq!(pattern.uses_global_counter(), global_counter);
        assert_eq!(pattern.uses_regex_capture(), regex_capture);
    }

    mod eval {
        use super::super::eval::{Context, Error, ErrorKind};
        use super::*;
        use crate::pattern::parser::ParsedItem;
        use test_case::test_case;

        #[test]
        fn err() {
            let pattern = Pattern::from(vec![Parsed::from(Item::Expression(vec![Parsed {
                value: Filter::CanonicalPath,
                range: 1..2,
            }]))]);
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Err(Error {
                    kind: ErrorKind::CanonicalizationFailed(AnyString::any()),
                    value: "dir/file.ext".into(),
                    cause: &Filter::CanonicalPath,
                    range: &(1..2usize),
                })
            );
        }

        #[test_case("",    constant(),      None, "abc"                 ; "constant ")]
        #[test_case("a/b", empty_expr(),    None, "a/b"                 ; "empty expression")]
        #[test_case("a/b", single_filter(), None, "b"                   ; "single filter ")]
        #[test_case("a/b", multi_filter(),  None, "B"                   ; "multi filter ")]
        #[test_case("a/b", complex_expr(),  None, "1 a 2 B 3"           ; "complex expression")]
        #[test_case("a/b", complex_expr(),  Some('\''), "1 'a' 2 'B' 3" ; "quoted complex expression")]
        fn ok(input: &str, items: Vec<ParsedItem>, quotes: Option<char>, output: &str) {
            let pattern = Pattern::from(items);
            let mut context = Context::fixture();
            context.expression_quotes = quotes;
            assert_eq!(pattern.eval(input, &context), Ok(output.into()));
        }

        fn constant() -> Vec<ParsedItem> {
            vec![Parsed::from(Item::Constant("abc".into()))]
        }

        fn empty_expr() -> Vec<ParsedItem> {
            vec![Parsed::from(Item::Expression(vec![]))]
        }

        fn single_filter() -> Vec<ParsedItem> {
            vec![Parsed::from(Item::Expression(vec![Parsed::from(
                Filter::FileName,
            )]))]
        }

        fn multi_filter() -> Vec<ParsedItem> {
            vec![Parsed::from(Item::Expression(vec![
                Parsed::from(Filter::FileName),
                Parsed::from(Filter::ToUppercase),
            ]))]
        }

        fn complex_expr() -> Vec<ParsedItem> {
            vec![
                Parsed::from(Item::Constant("1 ".into())),
                Parsed::from(Item::Expression(vec![Parsed::from(
                    Filter::ParentDirectory,
                )])),
                Parsed::from(Item::Constant(" 2 ".into())),
                Parsed::from(Item::Expression(vec![
                    Parsed::from(Filter::FileName),
                    Parsed::from(Filter::ToUppercase),
                ])),
                Parsed::from(Item::Constant(" 3".into())),
            ]
        }
    }
}
