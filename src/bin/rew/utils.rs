use std::fmt::{self, Debug};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct Empty;

#[derive(Debug, Clone)]
pub struct AnyString(pub String);

impl PartialEq for AnyString {
    fn eq(&self, _: &Self) -> bool {
        // This is only useful when comparing system error messages in tests,
        // because we cannot rely on a specific error message.
        true
    }
}

impl fmt::Display for AnyString {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

pub trait HasRange {
    fn range(&self) -> &Range<usize>;
}

#[cfg(test)]
mod tests {
    use super::*;

    mod any_string {
        use super::*;

        #[test]
        fn partial_eq() {
            assert_eq!(AnyString(String::from("a")), AnyString(String::from("a")));
            assert_eq!(AnyString(String::from("a")), AnyString(String::from("b")));
        }

        #[test]
        fn display() {
            assert_eq!(AnyString(String::from("abc")).to_string(), "abc");
        }
    }
}
