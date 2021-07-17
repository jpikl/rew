use std::ops::Range;

pub type ErrorRange = Range<usize>;

pub trait GetErrorRange {
    fn error_range(&self) -> &ErrorRange;
}
