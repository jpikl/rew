use crate::utils::str_from_utf8;
use std::io::{BufRead, Result};

pub enum Terminator {
    Newline { required: bool },
    Byte { value: u8, required: bool },
    None,
}

pub struct Splitter<I: BufRead> {
    input: I,
    terminator: Terminator,
    buffer: Vec<u8>,
}

impl<I: BufRead> Splitter<I> {
    pub fn new(input: I, terminator: Terminator) -> Self {
        Self {
            input,
            terminator,
            buffer: Vec::new(),
        }
    }

    pub fn read(&mut self) -> Result<Option<(&str, usize)>> {
        self.buffer.clear();

        let mut size = match self.terminator {
            Terminator::Newline { .. } => self.input.read_until(b'\n', &mut self.buffer)?,
            Terminator::Byte { value, .. } => self.input.read_until(value, &mut self.buffer)?,
            Terminator::None => self.input.read_to_end(&mut self.buffer)?,
        };

        if size > 0 {
            let orig_size = size;

            let valid = match &self.terminator {
                Terminator::Newline { required } => {
                    if size > 0 && self.buffer[size - 1] == b'\n' {
                        size -= 1;
                        if size > 0 && self.buffer[size - 1] == b'\r' {
                            size -= 1;
                        }
                        true
                    } else {
                        !required
                    }
                }
                Terminator::Byte { value, required } => {
                    if size > 0 && self.buffer[size - 1] == *value {
                        size -= 1;
                        true
                    } else {
                        !required
                    }
                }
                Terminator::None => true,
            };

            if valid {
                return str_from_utf8(&self.buffer[..size]).map(|str| Some((str, orig_size)));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unpack_io_error;
    use test_case::test_case;

    const NONE: Terminator = Terminator::None;
    const NL_REQ: Terminator = Terminator::Newline { required: true };
    const NL_OPT: Terminator = Terminator::Newline { required: false };
    const B0_REQ: Terminator = Terminator::Byte {
        value: 0,
        required: true,
    };
    const B0_OPT: Terminator = Terminator::Byte {
        value: 0,
        required: false,
    };

    #[test_case(NL_REQ, "",                   0, None;                    "newline required lf empty")]
    #[test_case(NL_REQ, "abc\0\n\0def",       0, Some(("abc\0", 5));      "newline required lf between 0")]
    #[test_case(NL_REQ, "abc\0\n\0def",       1, None;                    "newline required lf between 1")]
    #[test_case(NL_REQ, "abc\0\n\0def\n",     0, Some(("abc\0", 5));      "newline required lf between end 0")]
    #[test_case(NL_REQ, "abc\0\n\0def\n",     1, Some(("\0def", 5));      "newline required lf between end 1")]
    #[test_case(NL_REQ, "abc\0\n\0def\n",     2, None;                    "newline required lf between end 2")]
    #[test_case(NL_REQ, "\n\n",               0, Some(("", 1));           "newline required lf consecutive 0")]
    #[test_case(NL_REQ, "\n\n",               1, Some(("", 1));           "newline required lf consecutive 1")]
    #[test_case(NL_REQ, "\n\n",               2, None;                    "newline required lf consecutive 2")]
    #[test_case(NL_REQ, "abc\0\r\n\0def",     0, Some(("abc\0", 6));      "newline required cr lf between 0")]
    #[test_case(NL_REQ, "abc\0\r\n\0def",     1, None;                    "newline required cr lf between 1")]
    #[test_case(NL_REQ, "abc\0\r\n\0def\r\n", 0, Some(("abc\0", 6));      "newline required cr lf between end 0")]
    #[test_case(NL_REQ, "abc\0\r\n\0def\r\n", 1, Some(("\0def", 6));      "newline required cr lf between end 1")]
    #[test_case(NL_REQ, "abc\0\r\n\0def\r\n", 2, None;                    "newline required cr lf between end 2")]
    #[test_case(NL_REQ, "\r\n\n\r\n",         0, Some(("", 2));           "newline required cr lf consecutive 0")]
    #[test_case(NL_REQ, "\r\n\n\r\n",         1, Some(("", 1));           "newline required cr lf consecutive 1")]
    #[test_case(NL_REQ, "\r\n\n\r\n",         2, Some(("", 2));           "newline required cr lf consecutive 2")]
    #[test_case(NL_REQ, "\r\n\n\r\n",         3, None;                    "newline required cr lf consecutive 3")]
    #[test_case(NL_OPT, "",                   0, None;                    "newline optional lf empty")]
    #[test_case(NL_OPT, "abc\0\n\0def",       0, Some(("abc\0", 5));      "newline optional lf between 0")]
    #[test_case(NL_OPT, "abc\0\n\0def",       1, Some(("\0def", 4));      "newline optional lf between 1")]
    #[test_case(NL_OPT, "abc\0\n\0def",       2, None;                    "newline optional lf between 2")]
    #[test_case(NL_OPT, "abc\0\n\0def\n",     0, Some(("abc\0", 5));      "newline optional lf between end 0")]
    #[test_case(NL_OPT, "abc\0\n\0def\n",     1, Some(("\0def", 5));      "newline optional lf between end 1")]
    #[test_case(NL_OPT, "abc\0\n\0def\n",     2, None;                    "newline optional lf between end 2")]
    #[test_case(NL_OPT, "\n\n",               0, Some(("", 1));           "newline optional lf consecutive 0")]
    #[test_case(NL_OPT, "\n\n",               1, Some(("", 1));           "newline optional lf consecutive 1")]
    #[test_case(NL_OPT, "\n\n",               2, None;                    "newline optional lf consecutive 2")]
    #[test_case(NL_OPT, "abc\0\r\n\0def",     0, Some(("abc\0", 6));      "newline optional cr lf between 0")]
    #[test_case(NL_OPT, "abc\0\r\n\0def",     1, Some(("\0def", 4));      "newline optional cr lf between 1")]
    #[test_case(NL_OPT, "abc\0\r\n\0def",     2, None;                    "newline optional cr lf between 2")]
    #[test_case(NL_OPT, "abc\0\r\n\0def\r\n", 0, Some(("abc\0", 6));      "newline optional cr lf between end 0")]
    #[test_case(NL_OPT, "abc\0\r\n\0def\r\n", 1, Some(("\0def", 6));      "newline optional cr lf between end 1")]
    #[test_case(NL_OPT, "abc\0\r\n\0def\r\n", 2, None;                    "newline optional cr lf between end 2")]
    #[test_case(NL_OPT, "\r\n\n\r\n",         0, Some(("", 2));           "newline optional cr lf consecutive 0")]
    #[test_case(NL_OPT, "\r\n\n\r\n",         1, Some(("", 1));           "newline optional cr lf consecutive 1")]
    #[test_case(NL_OPT, "\r\n\n\r\n",         2, Some(("", 2));           "newline optional cr lf consecutive 2")]
    #[test_case(NL_OPT, "\r\n\n\r\n",         3, None;                    "newline optional cr lf consecutive 3")]
    #[test_case(B0_REQ, "",                   0, None;                    "byte required empty")]
    #[test_case(B0_REQ, "abc\n\0\ndef",       0, Some(("abc\n", 5));      "byte required between 0")]
    #[test_case(B0_REQ, "abc\n\0\ndef",       1, None;                    "byte required between 1")]
    #[test_case(B0_REQ, "abc\n\0\ndef\0",     0, Some(("abc\n", 5));      "byte required between end 0")]
    #[test_case(B0_REQ, "abc\n\0\ndef\0",     1, Some(("\ndef", 5));      "byte required between end 1")]
    #[test_case(B0_REQ, "abc\n\0\ndef\0",     2, None;                    "byte required between end 2")]
    #[test_case(B0_REQ, "\0\0",               0, Some(("", 1));           "byte required consecutive 0")]
    #[test_case(B0_REQ, "\0\0",               1, Some(("", 1));           "byte required consecutive 1")]
    #[test_case(B0_REQ, "\0\0",               2, None;                    "byte required consecutive 2")]
    #[test_case(B0_OPT, "",                   0, None;                    "byte optional empty")]
    #[test_case(B0_OPT, "abc\n\0\ndef",       0, Some(("abc\n", 5));      "byte optional between 0")]
    #[test_case(B0_OPT, "abc\n\0\ndef",       1, Some(("\ndef", 4));      "byte optional between 1")]
    #[test_case(B0_OPT, "abc\n\0\ndef",       2, None;                    "byte optional between 2")]
    #[test_case(B0_OPT, "abc\n\0\ndef\0",     0, Some(("abc\n", 5));      "byte optional between end 0")]
    #[test_case(B0_OPT, "abc\n\0\ndef\0",     1, Some(("\ndef", 5));      "byte optional between end 1")]
    #[test_case(B0_OPT, "abc\n\0\ndef\0",     2, None;                    "byte optional between end 2")]
    #[test_case(B0_OPT, "\0\0",               0, Some(("", 1));           "byte optional consecutive 0")]
    #[test_case(B0_OPT, "\0\0",               1, Some(("", 1));           "byte optional consecutive 1")]
    #[test_case(B0_OPT, "\0\0",               2, None;                    "byte optional consecutive 2")]
    #[test_case(NONE,   "",                   0, None;                    "none empty")]
    #[test_case(NONE,   "abc\n\0def",         0, Some(("abc\n\0def", 8)); "none nonempty 0")]
    #[test_case(NONE,   "abc\n\0def",         1, None;                    "none nonempty 1")]
    fn read(terminator: Terminator, input: &str, position: usize, result: Option<(&str, usize)>) {
        let mut splitter = Splitter::new(input.as_bytes(), terminator);
        for _ in 0..position {
            splitter.read().unwrap_or_default();
        }
        assert_eq!(splitter.read().map_err(unpack_io_error), Ok(result));
    }
}
