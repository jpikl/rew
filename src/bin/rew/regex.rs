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
    use claim::*;
    use regex::Match;

    mod path {
        use super::*;

        #[test]
        fn empty() {
            let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
            assert_none!(Solver::Value(&regex).eval(""));
        }

        #[test]
        fn nonempty() {
            let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
            let captures = Solver::Value(&regex).eval("dir_DIR/file_FILE.ext").unwrap();

            assert_eq!(captures.len(), 3);
            assert_eq!(captures.get(1).as_ref().map(Match::as_str), Some("dir"));
            assert_eq!(captures.get(2).as_ref().map(Match::as_str), Some("DIR"));
        }
    }

    mod file_name {
        use super::*;

        #[test]
        fn empty() {
            let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
            assert_none!(Solver::FileName(&regex).eval("/"));
        }

        #[test]
        fn nonempty() {
            let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
            let captures = Solver::FileName(&regex)
                .eval("dir_DIR/file_FILE.ext")
                .unwrap();

            assert_eq!(captures.len(), 3);
            assert_eq!(captures.get(1).as_ref().map(Match::as_str), Some("file"));
            assert_eq!(captures.get(2).as_ref().map(Match::as_str), Some("FILE"));
        }
    }

    #[test]
    fn none() {
        assert_none!(Solver::None.eval("abc"));
    }
}
