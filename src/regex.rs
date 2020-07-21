use regex::{Captures, Regex};
use std::path::Path;

pub struct Capture<'a> {
    regex: Option<&'a Regex>,
    full: bool,
}

impl<'a> Capture<'a> {
    pub fn of_full_path(regex: &'a Regex) -> Self {
        Self {
            regex: Some(regex),
            full: true,
        }
    }

    pub fn of_file_name(regex: &'a Regex) -> Self {
        Self {
            regex: Some(regex),
            full: false,
        }
    }

    pub fn of_none() -> Self {
        Self {
            regex: None,
            full: false,
        }
    }

    pub fn get<'t>(&self, path: &'t Path) -> Option<Captures<'t>> {
        self.regex
            .as_ref()
            .map(|regex| {
                if self.full {
                    path.file_name()
                        .map(|file_name| regex.captures(file_name.to_str().unwrap())) // TODO handle utf error
                        .flatten()
                } else {
                    regex.captures(path.to_str().unwrap()) // TODO handle utf error
                }
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_path() {
        // TODO test
    }

    #[test]
    fn file_name() {
        // TODO test
    }

    #[test]
    fn none() {
        // TODO test
    }
}
