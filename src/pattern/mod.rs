use crate::pattern::transform::Transform;
use crate::pattern::variable::Variable;

mod error;
mod lexer;
mod number;
mod range;
mod reader;
mod substitution;
mod transform;
mod variable;

#[derive(Debug, PartialEq)]
struct Pattern {
    items: Vec<Parsed<PatternItem>>,
}

#[derive(Debug, PartialEq)]
enum PatternItem {
    Constant(String),
    Expression {
        variable: Parsed<Variable>,
        transforms: Vec<Parsed<Transform>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    value: T,
    start: usize,
    end: usize,
}
