use crate::pattern::variable::Variable;
use crate::pattern::Parsed;

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub message: &'static str,
    pub position: usize,
}

#[derive(Debug, PartialEq)]
pub struct EvalError<'a> {
    pub message: &'static str,
    pub variable: &'a Parsed<Variable>,
}
