use regex::{Captures, Regex};
use std::path::Path;

pub enum Solver<'a> {
    Filename(&'a Regex),
    FullPath(&'a Regex),
    None,
}

impl<'a> Solver<'a> {
    pub fn eval<'t>(&self, path: &'t Path) -> Option<Captures<'t>> {
        match self {
            Self::Filename(regex) => path
                .file_name()
                .map(|file_name| regex.captures(file_name.to_str().unwrap())) // TODO handle utf error
                .flatten(),
            Self::FullPath(regex) => regex.captures(path.to_str().unwrap()), // TODO handle utf error,
            Self::None => None,
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
        solver.eval(&Path::new("dir_DIR/file_FILE.ext"))
    }

    #[test]
    fn none() {
        assert!(eval(Solver::None).is_none());
    }
}
