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
    use test_case::test_case;

    #[test_case(Solver::Value(&regex()), ""; "value empty")]
    #[test_case(Solver::Value(&regex()), "ab/cd"; "value nonempty")]
    #[test_case(Solver::FileName(&regex()), ""; "file name empty")]
    #[test_case(Solver::FileName(&regex()), "ab/cd"; "file name nonempty")]
    #[test_case(Solver::None, ""; "none empty")]
    #[test_case(Solver::None, "ab/cd"; "none nonempty")]
    fn missed(solver: Solver, input: &str) {
        assert_none!(solver.eval(input));
    }

    #[test_case(Solver::Value(&regex()), "aB/cD", 0, Some("aB"); "value group 0")]
    #[test_case(Solver::Value(&regex()), "aB/cD", 1, Some("a"); "value group 1")]
    #[test_case(Solver::Value(&regex()), "aB/cD", 2, Some("B"); "value group 2")]
    #[test_case(Solver::Value(&regex()), "aB/cD", 3, None; "value group 3")]
    #[test_case(Solver::FileName(&regex()), "aB/cD", 0, Some("cD"); "file name group 0")]
    #[test_case(Solver::FileName(&regex()), "aB/cD", 1, Some("c"); "file name group 1")]
    #[test_case(Solver::FileName(&regex()), "aB/cD", 2, Some("D"); "file name group 2")]
    #[test_case(Solver::FileName(&regex()), "aB/cD", 3, None; "file name group 3")]
    fn captured(solver: Solver, input: &str, group: usize, result: Option<&str>) {
        assert_eq!(
            solver
                .eval(input)
                .unwrap()
                .get(group)
                .as_ref()
                .map(Match::as_str),
            result
        );
    }

    fn regex() -> Regex {
        Regex::new("([a-z])([A-Z])").unwrap()
    }
}
