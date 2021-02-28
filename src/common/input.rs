use crate::utils::str_from_utf8;
use std::io::{BufRead, Result};

pub enum Terminator {
    Newline,
    Byte(u8),
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
            Terminator::Newline => self.input.read_until(b'\n', &mut self.buffer)?,
            Terminator::Byte(terminator) => self.input.read_until(terminator, &mut self.buffer)?,
            Terminator::None => self.input.read_to_end(&mut self.buffer)?,
        };

        if size > 0 {
            let orig_size = size;

            match self.terminator {
                Terminator::Newline => {
                    if size > 0 && self.buffer[size - 1] == b'\n' {
                        size -= 1;
                        if size > 0 && self.buffer[size - 1] == b'\r' {
                            size -= 1;
                        }
                    }
                }
                Terminator::Byte(terminator) => {
                    if size > 0 && self.buffer[size - 1] == terminator {
                        size -= 1;
                    }
                }
                Terminator::None => {}
            }

            str_from_utf8(&self.buffer[..size]).map(|str| Some((str, orig_size)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::unpack_io_error;

    mod newline_terminator {
        use super::*;

        #[test]
        fn empty() {
            let mut splitter = Splitter::new(&[][..], Terminator::Newline);
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        mod lf_only {
            use super::*;

            #[test]
            fn between() {
                let mut splitter = Splitter::new(&b"abc\0\n\0def"[..], Terminator::Newline);
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("abc\0", 5)))
                );
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("\0def", 4)))
                );
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn between_and_end() {
                let mut splitter = Splitter::new(&b"abc\0\n\0def\n"[..], Terminator::Newline);
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("abc\0", 5)))
                );
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("\0def", 5)))
                );
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn consecutive() {
                let mut splitter = Splitter::new(&b"\n\n"[..], Terminator::Newline);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }
        }

        mod cr_and_lf {
            use super::*;

            #[test]
            fn between() {
                let mut splitter = Splitter::new(&b"abc\0\r\n\0def"[..], Terminator::Newline);
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("abc\0", 6)))
                );
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("\0def", 4)))
                );
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn between_and_end() {
                let mut splitter = Splitter::new(&b"abc\0\r\n\0def\r\n"[..], Terminator::Newline);
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("abc\0", 6)))
                );
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("\0def", 6)))
                );
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn consecutive() {
                let mut splitter = Splitter::new(&b"\r\n\n\r\n"[..], Terminator::Newline);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }
        }
    }

    mod byte_terminator {
        use super::*;

        #[test]
        fn empty() {
            let mut splitter = Splitter::new(&[][..], Terminator::Byte(0));
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn between() {
            let mut splitter = Splitter::new(&b"abc\n\0\ndef"[..], Terminator::Byte(0));
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("abc\n", 5)))
            );
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("\ndef", 4)))
            );
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn between_and_end() {
            let mut splitter = Splitter::new(&b"abc\n\0\ndef\0"[..], Terminator::Byte(0));
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("abc\n", 5)))
            );
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("\ndef", 5)))
            );
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn consecutive() {
            let mut splitter = Splitter::new(&b"\0\0"[..], Terminator::Byte(0));
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }
    }

    mod no_terminator {
        use super::*;

        #[test]
        fn empty() {
            let mut splitter = Splitter::new(&[][..], Terminator::None);
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn nonempty() {
            let mut splitter = Splitter::new(&b"abc\n\0def"[..], Terminator::None);
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("abc\n\0def", 8)))
            );
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }
    }
}
