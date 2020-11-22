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

    #[test]
    fn match_in_some() {
        assert_eq!(
            get_match(String::from("abc123def456"), &Regex::new("\\d+").unwrap()),
            Ok(String::from("123"))
        );
    }
    #[test]
    fn match_in_empty() {
        assert_eq!(
            get_match(String::new(), &Regex::new("\\d+").unwrap()),
            Ok(String::new(),)
        );
    }

    #[test]
    fn replace_first_in_some() {
        assert_eq!(
            replace_first(
                String::from("abc123def456"),
                &Regex::new("(\\d)(\\d+)").unwrap(),
                "_$2$1_"
            ),
            Ok(String::from("abc_231_def456"))
        );
    }

    #[test]
    fn replace_first_in_empty() {
        assert_eq!(
            replace_first(String::new(), &Regex::new("(\\d)(\\d+)").unwrap(), "_$2$1_"),
            Ok(String::new())
        );
    }

    #[test]
    fn replace_all_in_some() {
        assert_eq!(
            replace_all(
                String::from("abc123def456"),
                &Regex::new("(\\d)(\\d+)").unwrap(),
                "_$2$1_"
            ),
            Ok(String::from("abc_231_def_564_"))
        );
    }

    #[test]
    fn replace_all_in_empty() {
        assert_eq!(
            replace_all(String::new(), &Regex::new("(\\d)(\\d+)").unwrap(), "_$2$1_"),
            Ok(String::new())
        );
    }

    #[test]
    fn remove_first_in_some() {
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
    fn remove_first_in_empty() {
        assert_eq!(
            replace_first(String::new(), &Regex::new("\\d+").unwrap(), ""),
            Ok(String::new())
        );
    }

    #[test]
    fn remove_all_in_some() {
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
    fn remove_all_in_empty() {
        assert_eq!(
            replace_all(String::new(), &Regex::new("\\d+").unwrap(), ""),
            Ok(String::new())
        );
    }

    #[test]
    fn get_capture_from_none() {
        assert_eq!(get_capture(None, 1), Ok(String::new()));
    }

    #[test]
    fn get_capture_from_some() {
        assert_eq!(
            get_capture(make_regex_captures().as_ref(), 1),
            Ok(String::from("abc"))
        );
    }

    #[test]
    fn get_capture_from_some_wrong_number() {
        assert_eq!(
            get_capture(make_regex_captures().as_ref(), 2),
            Ok(String::new())
        );
    }

    #[test]
    fn adds_capture_group_brackets() {
        assert_eq!(add_capture_group_brackets("ab"), "ab");
        assert_eq!(add_capture_group_brackets("a$1b"), "a${1}b");
        assert_eq!(
            add_capture_group_brackets("$1a$12b$123"),
            "${1}a${12}b${123}"
        );
    }
}
