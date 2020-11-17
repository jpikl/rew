use crate::pattern::filter::error::Result;
use regex::Regex;

pub fn assert_ok_uuid(result: Result) {
    let value = result.expect("Expected Ok result");
    let regex_str = "^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
    let regex = Regex::new(regex_str).unwrap();
    assert!(regex.is_match(&value), format!("{} is UUID v4", value));
}