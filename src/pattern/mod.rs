use crate::pattern::parse::Parsed;
use crate::pattern::transform::Transform;
use crate::pattern::variable::Variable;

mod error;
mod eval;
mod lexer;
mod number;
mod parse;
mod parser;
mod range;
mod reader;
mod substitution;
mod transform;
mod variable;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    items: Vec<Parsed<PatternItem>>,
}

#[derive(Debug, PartialEq)]
pub enum PatternItem {
    Constant(String),
    Expression {
        variable: Parsed<Variable>,
        transforms: Vec<Parsed<Transform>>,
    },
}
