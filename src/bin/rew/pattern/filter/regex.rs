use crate::pattern::filter::error::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;

lazy_static! {
    static ref CAPTURE_GROUP_VAR_REGEX: Regex = Regex::new(r"\$(\d+)").unwrap();
}

pub fn get_match(value: String, regex: &Regex) -> Result {
    match regex.find(&value) {
        Some(result) => Ok(result.as_str().to_string()),
        None => Ok(String::new()),
    }
}

pub fn replace_first(value: String, regex: &Regex, replacement: &str) -> Result {
    let replacement = add_capture_group_brackets(replacement);
    Ok(regex.replacen(&value, 1, replacement.as_ref()).to_string())
}

pub fn replace_all(value: String, regex: &Regex, replacement: &str) -> Result {
    let replacement = add_capture_group_brackets(replacement);
    Ok(regex.replace_all(&value, replacement.as_ref()).to_string())
}

pub fn get_capture(captures: Option<&regex::Captures>, number: usize) -> Result {
    Ok(captures
        .map(|captures| captures.get(number))
        .flatten()
        .map(|capture| capture.as_str())
        .map_or_else(String::new, String::from))
}

fn add_capture_group_brackets(string: &str) -> Cow<str> {
    if string.contains('$') {
        CAPTURE_GROUP_VAR_REGEX.replace_all(string, r"$${${1}}")
    } else {
        Cow::Borrowed(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::testing::make_regex_captures;

    mod get_match {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(
                get_match(String::new(), &Regex::new("\\d+").unwrap()),
                Ok(String::new())
            );
        }

        #[test]
        fn none() {
            assert_eq!(
                get_match(String::from("abc"), &Regex::new("\\d+").unwrap()),
                Ok(String::new())
            );
        }

        #[test]
        fn first() {
            assert_eq!(
                get_match(String::from("abc123def456"), &Regex::new("\\d+").unwrap()),
                Ok(String::from("123"))
            );
        }
    }

    mod replace_first {
        use super::*;

        #[test]
        fn empty_with_empty() {
            assert_eq!(
                replace_first(String::new(), &Regex::new("\\d+").unwrap(), ""),
                Ok(String::new())
            );
        }

        #[test]
        fn empty_with_nonempty() {
            assert_eq!(
                replace_first(String::new(), &Regex::new("(\\d)(\\d+)").unwrap(), "_$2$1_"),
                Ok(String::new())
            );
        }

        #[test]
        fn none_with_empty() {
            assert_eq!(
                replace_first(String::from("abc"), &Regex::new("\\d+").unwrap(), ""),
                Ok(String::from("abc"))
            );
        }

        #[test]
        fn none_with_nonempty() {
            assert_eq!(
                replace_first(
                    String::from("abc"),
                    &Regex::new("(\\d)(\\d+)").unwrap(),
                    "_$2$1_"
                ),
                Ok(String::from("abc"))
            );
        }

        #[test]
        fn first_with_empty() {
            assert_eq!(
                replace_first(
                    String::from("abc123def456"),
                    &Regex::new("\\d+").unwrap(),
                    ""
                ),
                Ok(String::from("abcdef456"))
            );
        }

        #[test]
        fn first_with_nonempty() {
            assert_eq!(
                replace_first(
                    String::from("abc123def456"),
                    &Regex::new("(\\d)(\\d+)").unwrap(),
                    "_$2$1_"
                ),
                Ok(String::from("abc_231_def456"))
            );
        }
    }

    mod replace_all {
        use super::*;

        #[test]
        fn empty_with_empty() {
            assert_eq!(
                replace_all(String::new(), &Regex::new("\\d+").unwrap(), ""),
                Ok(String::new())
            );
        }

        #[test]
        fn empty_with_nonempty() {
            assert_eq!(
                replace_all(String::new(), &Regex::new("(\\d)(\\d+)").unwrap(), "_$2$1_"),
                Ok(String::new())
            );
        }

        #[test]
        fn none_with_empty() {
            assert_eq!(
                replace_all(String::from("abc"), &Regex::new("\\d+").unwrap(), ""),
                Ok(String::from("abc"))
            );
        }

        #[test]
        fn none_with_nonempty() {
            assert_eq!(
                replace_all(
                    String::from("abc"),
                    &Regex::new("(\\d)(\\d+)").unwrap(),
                    "_$2$1_"
                ),
                Ok(String::from("abc"))
            );
        }

        #[test]
        fn all_with_empty() {
            assert_eq!(
                replace_all(
                    String::from("abc123def456"),
                    &Regex::new("\\d+").unwrap(),
                    ""
                ),
                Ok(String::from("abcdef"))
            );
        }

        #[test]
        fn all_with_nonempty() {
            assert_eq!(
                replace_all(
                    String::from("abc123def456"),
                    &Regex::new("(\\d)(\\d+)").unwrap(),
                    "_$2$1_"
                ),
                Ok(String::from("abc_231_def_564_"))
            );
        }
    }

    mod get_capture {
        use super::*;

        #[test]
        fn none() {
            assert_eq!(get_capture(None, 1), Ok(String::new()));
        }

        #[test]
        fn some() {
            assert_eq!(
                get_capture(make_regex_captures().as_ref(), 1),
                Ok(String::from("abc"))
            );
        }

        #[test]
        fn some_invalid() {
            assert_eq!(
                get_capture(make_regex_captures().as_ref(), 2),
                Ok(String::new())
            );
        }
    }

    mod add_capture_group_brackets {
        use super::*;

        #[test]
        fn zero() {
            assert_eq!(add_capture_group_brackets("ab"), "ab");
        }

        #[test]
        fn one() {
            assert_eq!(add_capture_group_brackets("a$1b"), "a${1}b");
        }

        #[test]
        fn multiple() {
            assert_eq!(
                add_capture_group_brackets("$1a$12b$123"),
                "${1}a${12}b${123}"
            );
        }
    }
}
