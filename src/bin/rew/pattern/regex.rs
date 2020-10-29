use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::fmt;

lazy_static! {
    static ref CAPTURE_GROUP_VAR_REGEX: Regex = Regex::new(r"\$([0-9]+)").unwrap();
}

#[derive(Debug)]
pub struct RegexHolder(pub Regex);

impl PartialEq for RegexHolder {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl fmt::Display for RegexHolder {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

pub fn add_capture_group_brackets(string: &str) -> Cow<str> {
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
    fn regex_holder_eq() {
        assert_eq!(
            RegexHolder(Regex::new("[a-z]+").unwrap()),
            RegexHolder(Regex::new("[a-z]+").unwrap())
        );
        assert_ne!(
            RegexHolder(Regex::new("[a-z]+").unwrap()),
            RegexHolder(Regex::new("[a-z]*").unwrap())
        );
    }

    #[test]
    fn regex_holder_fmt() {
        assert_eq!(
            RegexHolder(Regex::new("[a-z]+").unwrap()).to_string(),
            String::from("[a-z]+")
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
