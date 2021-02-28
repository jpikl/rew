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

    mod newline_terminator {
        use super::*;

        mod required {
            use super::*;

            const TERMINATOR: Terminator = Terminator::Newline { required: true };

            #[test]
            fn empty() {
                let mut splitter = Splitter::new(&[][..], TERMINATOR);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            mod lf_only {
                use super::*;

                #[test]
                fn between() {
                    let mut splitter = Splitter::new(&b"abc\0\n\0def"[..], TERMINATOR);
                    assert_eq!(
                        splitter.read().map_err(unpack_io_error),
                        Ok(Some(("abc\0", 5)))
                    );
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
                }

                #[test]
                fn between_and_end() {
                    let mut splitter = Splitter::new(&b"abc\0\n\0def\n"[..], TERMINATOR);
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
                    let mut splitter = Splitter::new(&b"\n\n"[..], TERMINATOR);
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
                }
            }

            mod cr_and_lf {
                use super::*;

                #[test]
                fn between() {
                    let mut splitter = Splitter::new(&b"abc\0\r\n\0def"[..], TERMINATOR);
                    assert_eq!(
                        splitter.read().map_err(unpack_io_error),
                        Ok(Some(("abc\0", 6)))
                    );
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
                }

                #[test]
                fn between_and_end() {
                    let mut splitter = Splitter::new(&b"abc\0\r\n\0def\r\n"[..], TERMINATOR);
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
                    let mut splitter = Splitter::new(&b"\r\n\n\r\n"[..], TERMINATOR);
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
                }
            }
        }

        mod optional {
            use super::*;

            const TERMINATOR: Terminator = Terminator::Newline { required: false };

            #[test]
            fn empty() {
                let mut splitter = Splitter::new(&[][..], TERMINATOR);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            mod lf_only {
                use super::*;

                #[test]
                fn between() {
                    let mut splitter = Splitter::new(&b"abc\0\n\0def"[..], TERMINATOR);
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
                    let mut splitter = Splitter::new(&b"abc\0\n\0def\n"[..], TERMINATOR);
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
                    let mut splitter = Splitter::new(&b"\n\n"[..], TERMINATOR);
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
                }
            }

            mod cr_and_lf {
                use super::*;

                #[test]
                fn between() {
                    let mut splitter = Splitter::new(&b"abc\0\r\n\0def"[..], TERMINATOR);
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
                    let mut splitter = Splitter::new(&b"abc\0\r\n\0def\r\n"[..], TERMINATOR);
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
                    let mut splitter = Splitter::new(&b"\r\n\n\r\n"[..], TERMINATOR);
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 2))));
                    assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
                }
            }
        }
    }

    mod byte_terminator {
        use super::*;

        mod required {
            use super::*;

            const TERMINATOR: Terminator = Terminator::Byte {
                value: 0,
                required: true,
            };

            #[test]
            fn empty() {
                let mut splitter = Splitter::new(&[][..], TERMINATOR);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn between() {
                let mut splitter = Splitter::new(&b"abc\n\0\ndef"[..], TERMINATOR);
                assert_eq!(
                    splitter.read().map_err(unpack_io_error),
                    Ok(Some(("abc\n", 5)))
                );
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn between_and_end() {
                let mut splitter = Splitter::new(&b"abc\n\0\ndef\0"[..], TERMINATOR);
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
                let mut splitter = Splitter::new(&b"\0\0"[..], TERMINATOR);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }
        }

        mod optional {
            use super::*;

            const TERMINATOR: Terminator = Terminator::Byte {
                value: 0,
                required: false,
            };

            #[test]
            fn empty() {
                let mut splitter = Splitter::new(&[][..], TERMINATOR);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }

            #[test]
            fn between() {
                let mut splitter = Splitter::new(&b"abc\n\0\ndef"[..], TERMINATOR);
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
                let mut splitter = Splitter::new(&b"abc\n\0\ndef\0"[..], TERMINATOR);
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
                let mut splitter = Splitter::new(&b"\0\0"[..], TERMINATOR);
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(Some(("", 1))));
                assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
            }
        }
    }

    mod no_terminator {
        use super::*;

        const TERMINATOR: Terminator = Terminator::None;

        #[test]
        fn empty() {
            let mut splitter = Splitter::new(&[][..], TERMINATOR);
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn nonempty() {
            let mut splitter = Splitter::new(&b"abc\n\0def"[..], TERMINATOR);
            assert_eq!(
                splitter.read().map_err(unpack_io_error),
                Ok(Some(("abc\n\0def", 8)))
            );
            assert_eq!(splitter.read().map_err(unpack_io_error), Ok(None));
        }
    }
}
