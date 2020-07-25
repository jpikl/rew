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
        CAPTURE_GROUP_VAR_REGEX.replace_all(string, r"${${1}}")
    } else {
        Cow::Borrowed(string)
    }
}
