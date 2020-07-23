use regex::{Captures, Regex};
use std::path::Path;
use std::{error, fmt};

#[derive(Debug)]
pub struct Utf8Error {}

impl Utf8Error {
    pub fn new() -> Self {
        Self {}
    }
}

impl error::Error for Utf8Error {}

impl fmt::Display for Utf8Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Input does not have UTF-8 encoding")
    }
}

pub enum Solver<'a> {
    Filename(&'a Regex),
    FullPath(&'a Regex),
    None,
}

impl<'a> Solver<'a> {
    pub fn eval<'t>(&self, path: &'t Path) -> Result<Option<Captures<'t>>, Utf8Error> {
        match self {
            Self::Filename(regex) => {
                if let Some(filename) = path.file_name() {
                    Ok(regex.captures(filename.to_str().ok_or_else(Utf8Error::new)?))
                } else {
                    Ok(None)
                }
            }
            Self::FullPath(regex) => Ok(regex.captures(path.to_str().ok_or_else(Utf8Error::new)?)),
            Self::None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Match;

    #[test]
    fn file_name() {
        let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
        let captures = eval(Solver::Filename(&regex)).unwrap();

        assert_eq!(captures.len(), 3);
        assert_eq!(captures.get(1).as_ref().map(Match::as_str), Some("file"));
        assert_eq!(captures.get(2).as_ref().map(Match::as_str), Some("FILE"));
    }

    #[test]
    fn full_path() {
        let regex = Regex::new("([a-z]+)_([A-Z]+)").unwrap();
        let captures = eval(Solver::FullPath(&regex)).unwrap();

        assert_eq!(captures.len(), 3);
        assert_eq!(captures.get(1).as_ref().map(Match::as_str), Some("dir"));
        assert_eq!(captures.get(2).as_ref().map(Match::as_str), Some("DIR"));
    }

    fn eval(solver: Solver) -> Option<Captures> {
        solver.eval(&Path::new("dir_DIR/file_FILE.ext")).unwrap()
    }

    #[test]
    fn none() {
        assert!(eval(Solver::None).is_none());
    }
}
