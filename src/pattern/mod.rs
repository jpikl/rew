pub use crate::pattern::eval::EvalContext;
use crate::pattern::parse::Parsed;
use crate::pattern::parser::PatternItem;

mod char;
mod error;
mod eval;
mod lexer;
mod number;
mod parse;
mod parser;
mod range;
mod reader;
mod render;
mod substitution;
mod transform;
mod variable;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    items: Vec<Parsed<PatternItem>>,
}
