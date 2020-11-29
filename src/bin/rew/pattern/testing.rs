use crate::pattern::eval;
use crate::pattern::parse::Parsed;
use regex::{Captures, Regex};
use std::path::Path;

pub fn make_eval_context<'a>() -> eval::Context<'a> {
    eval::Context {
        #[cfg(unix)]
        working_dir: Path::new("/work"),
        #[cfg(windows)]
        working_dir: Path::new("C:\\work"),
        local_counter: 1,
        global_counter: 2,
        regex_captures: make_regex_captures(),
    }
}

pub fn make_regex_captures<'a>() -> Option<Captures<'a>> {
    Regex::new("(.*)").unwrap().captures("abc")
}

pub fn make_parsed<T>(value: T) -> Parsed<T> {
    Parsed { value, range: 0..0 }
}
