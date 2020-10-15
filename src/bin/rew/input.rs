use common::input::{Delimiter, Splitter};
use common::io::Input;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::slice::Iter;

pub enum Paths<'a, I: Input> {
    Args { iter: Iter<'a, PathBuf> },
    Stdin { splitter: Splitter<I> },
}

impl<'a, I: Input> Paths<'a, I> {
    pub fn from_args(values: &'a [PathBuf]) -> Self {
        Paths::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(stdin: I, delimiter: Delimiter) -> Self {
        Paths::Stdin {
            splitter: Splitter::new(stdin, delimiter),
        }
    }

    pub fn next(&mut self) -> Result<Option<&Path>> {
        match self {
            Self::Args { iter } => Ok(iter.next().map(PathBuf::as_path)),
            Self::Stdin { splitter: reader } => reader.read().map(|opt_str| opt_str.map(Path::new)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::io::mem::MemoryInput;
    use std::io::{Error, ErrorKind};

    #[test]
    fn paths_from_args() {
        let args = vec![
            PathBuf::from(String::from("a")),
            PathBuf::from(String::from("b")),
        ];
        let mut paths: Paths<MemoryInput> = Paths::from_args(&args);
        assert_eq!(paths.next().map_err(map_err), Ok(Some(Path::new("a"))));
        assert_eq!(paths.next().map_err(map_err), Ok(Some(Path::new("b"))));
        assert_eq!(paths.next().map_err(map_err), Ok(None));
    }

    #[test]
    fn paths_from_stdin() {
        let stdin = MemoryInput::new(&b"a\nb"[..]);
        let mut paths: Paths<MemoryInput> = Paths::from_stdin(stdin, Delimiter::Newline);
        assert_eq!(paths.next().map_err(map_err), Ok(Some(Path::new("a"))));
        assert_eq!(paths.next().map_err(map_err), Ok(Some(Path::new("b"))));
        assert_eq!(paths.next().map_err(map_err), Ok(None));
    }

    fn map_err(error: Error) -> (ErrorKind, String) {
        (error.kind(), error.to_string())
    }
}
