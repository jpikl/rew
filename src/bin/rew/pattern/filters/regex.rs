use crate::pattern::filters::error::Result;
use crate::pattern::regex::RegexHolder;
use crate::pattern::substitution::Substitution;
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;

lazy_static! {
    static ref CAPTURE_GROUP_VAR_REGEX: Regex = Regex::new(r"\$([0-9]+)").unwrap();
}

pub fn get_match(mut value: String, regex: &RegexHolder) -> Result {
    match regex.0.find(&value) {
        Some(result) => Ok(result.as_str().to_string()),
        None => Ok(String::new()),
    }
}

pub fn replace_first(mut value: String, subst: &Substitution<RegexHolder>) -> Result {
    let regex = &subst.value.0;
    let replacement = add_capture_group_brackets(&subst.replacement);
    Ok(regex.replacen(&value, 1, replacement.as_ref()).to_string())
}

pub fn replace_all(mut value: String, subst: &Substitution<RegexHolder>) -> Result {
    let regex = &subst.value.0;
    let replacement = add_capture_group_brackets(&subst.replacement);
    Ok(regex.replace_all(&value, replacement.as_ref()).to_string())
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

    #[test]
    fn match_in_some() {
        assert_eq!(
            get_match(
                String::from("abc123def456"),
                &RegexHolder(Regex::new("[0-9]+").unwrap())
            ),
            Ok(String::from("123"))
        );
    }
    #[test]
    fn match_in_empty() {
        assert_eq!(
            get_match(String::new(), &RegexHolder(Regex::new("[0-9]+").unwrap())),
            Ok(String::new(),)
        );
    }

    #[test]
    fn replace_first_in_some() {
        assert_eq!(
            replace_first(
                String::from("abc123def456"),
                &Substitution {
                    value: RegexHolder(Regex::new("([0-9])([0-9]+)").unwrap()),
                    replacement: String::from("_$2$1_"),
                }
            ),
            Ok(String::from("abc_231_def456"))
        );
    }

    #[test]
    fn replace_first_in_empty() {
        assert_eq!(
            replace_first(
                String::new(),
                &Substitution {
                    value: RegexHolder(Regex::new("([0-9])([0-9]+)").unwrap()),
                    replacement: String::from("_$2$1_"),
                }
            ),
            Ok(String::new())
        );
    }

    #[test]
    fn replace_all_in_some() {
        assert_eq!(
            replace_all(
                String::from("abc123def456"),
                &Substitution {
                    value: RegexHolder(Regex::new("([0-9])([0-9]+)").unwrap()),
                    replacement: String::from("_$2$1_"),
                }
            ),
            Ok(String::from("abc_231_def_564_"))
        );
    }

    #[test]
    fn replace_all_in_empty() {
        assert_eq!(
            replace_all(
                String::new(),
                &Substitution {
                    value: RegexHolder(Regex::new("([0-9])([0-9]+)").unwrap()),
                    replacement: String::from("_$2$1_"),
                }
            ),
            Ok(String::new())
        );
    }

    #[test]
    fn remove_first_in_some() {
        assert_eq!(
            replace_first(
                String::from("abc123def456"),
                &Substitution {
                    value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                    replacement: String::new(),
                }
            ),
            Ok(String::from("abcdef456"))
        );
    }

    #[test]
    fn remove_first_in_empty() {
        assert_eq!(
            replace_first(
                String::new(),
                &Substitution {
                    value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                    replacement: String::new(),
                }
            ),
            Ok(String::new())
        );
    }

    #[test]
    fn remove_all_in_some() {
        assert_eq!(
            replace_all(
                String::from("abc123def456"),
                &Substitution {
                    value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                    replacement: String::new(),
                }
            ),
            Ok(String::from("abcdef"))
        );
    }

    #[test]
    fn remove_all_in_empty() {
        assert_eq!(
            replace_all(
                String::new(),
                &Substitution {
                    value: RegexHolder(Regex::new("[0-9]+").unwrap()),
                    replacement: String::new(),
                }
            ),
            Ok(String::new())
        );
    }

    #[test]
    fn adds_capture_group_brackets() {
        assert_eq!(add_capture_group_brackets("ab"), String::from("ab"));
        assert_eq!(add_capture_group_brackets("a$1b"), String::from("a${1}b"));
        assert_eq!(
            add_capture_group_brackets("$1a$12b$123"),
            String::from("${1}a${12}b${123}")
        );
    }
}
