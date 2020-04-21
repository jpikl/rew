pub use crate::pattern::eval::EvalContext;
use crate::pattern::lexer::Parsed;
pub use crate::pattern::lexer::{Lexer, DEFAULT_ESCAPE, META_CHARS};
pub use crate::pattern::parser::Parser;
use crate::pattern::parser::PatternItem;

mod char;
mod error;
mod eval;
mod lexer;
mod number;
mod parser;
mod query;
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

impl Pattern {
    pub fn new(items: Vec<Parsed<PatternItem>>) -> Self {
        Self { items }
    }
}
