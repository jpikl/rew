use std::fmt::{self, Debug};
use std::ops::Range;

pub trait HasRange {
    fn range(&self) -> &Range<usize>;
}

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

#[cfg(test)]
impl From<&str> for AnyString {
    fn from(value: &str) -> Self {
        Self(String::from(value))
    }
}

#[cfg(test)]
impl AnyString {
    pub fn any() -> Self {
        Self::from("This value is not compared by test assertions")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod any_string {
        use super::*;
        use test_case::test_case;

        #[test_case("a", "a"; "same")]
        #[test_case("a", "b"; "different")]
        fn partial_eq(left: &str, right: &str) {
            assert_eq!(AnyString::from(left), AnyString::from(right));
        }

        #[test]
        fn display() {
            assert_eq!(AnyString::from("abc").to_string(), "abc");
        }
    }
}
