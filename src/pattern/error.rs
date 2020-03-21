#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub message: &'static str,
    pub position: usize,
}
