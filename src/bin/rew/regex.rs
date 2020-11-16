use regex::{Captures, Regex};
use std::path::Path;

pub enum Solver<'a> {
    Value(&'a Regex),
    FileName(&'a Regex),
    None,
}

impl<'a> Solver<'a> {
    pub fn eval<'t>(&self, value: &'t str) -> Option<Captures<'t>> {
        match self {
            Self::Value(regex) => regex.captures(value),
            Self::FileName(regex) => {
                if let Some(file_name) = Path::new(value).file_name() {
                    regex.captures(
                        file_name
                            .to_str()
                            .expect("Expected file name to be in UTF-8"), // Because input is also in UTF-8
                    )
                } else {
                    None
                }
            }
            Self::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Match;

    #[test]
    fn path() {
        let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
        let captures = Solver::Value(&regex).eval("dir_DIR/file_FILE.ext").unwrap();

        assert_eq!(captures.len(), 3);
        assert_eq!(captures.get(1).as_ref().map(Match::as_str), Some("dir"));
        assert_eq!(captures.get(2).as_ref().map(Match::as_str), Some("DIR"));
    }

    #[test]
    fn path_empty() {
        let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
        assert_eq!(Solver::Value(&regex).eval("").is_none(), true);
    }

    #[test]
    fn file_name() {
        let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
        let captures = Solver::FileName(&regex)
            .eval("dir_DIR/file_FILE.ext")
            .unwrap();

        assert_eq!(captures.len(), 3);
        assert_eq!(captures.get(1).as_ref().map(Match::as_str), Some("file"));
        assert_eq!(captures.get(2).as_ref().map(Match::as_str), Some("FILE"));
    }

    #[test]
    fn file_name_empty() {
        let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
        assert_eq!(Solver::FileName(&regex).eval("/").is_none(), true);
    }

    #[test]
    fn none() {
        assert_eq!(Solver::None.eval("abc").is_none(), true);
    }
}
