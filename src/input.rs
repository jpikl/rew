use std::io::{BufRead, Error, ErrorKind, Result};

pub enum Delimiter {
    Newline,
    Nul,
    None,
}

pub struct Reader<T: BufRead> {
    reader: T,
    delimiter: Delimiter,
    buffer: Vec<u8>,
}

impl<T: BufRead> Reader<T> {
    pub fn new(reader: T, delimiter: Delimiter) -> Self {
        Self {
            reader,
            delimiter,
            buffer: Vec::new(),
        }
    }

    pub fn read(&mut self) -> Result<Option<&str>> {
        self.buffer.clear();

        let result = match self.delimiter {
            Delimiter::Newline => self.reader.read_until(b'\n', &mut self.buffer),
            Delimiter::Nul => self.reader.read_until(0, &mut self.buffer),
            Delimiter::None => self.reader.read_to_end(&mut self.buffer),
        };

        match result {
            Ok(0) => Ok(None),
            Ok(mut size) => {
                match self.delimiter {
                    Delimiter::Newline => {
                        if self.buffer[size - 1] == b'\n' {
                            size -= 1;
                            if self.buffer[size - 1] == b'\r' {
                                size -= 1;
                            }
                        }
                    }
                    Delimiter::Nul => {
                        if self.buffer[size - 1] == 0 {
                            size -= 1;
                        }
                    }
                    Delimiter::None => {}
                }
                match std::str::from_utf8(&self.buffer[..size]) {
                    Ok(str) => Ok(Some(str)),
                    Err(error) => Err(Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Input does not have UTF-8 encoding (offset: {})",
                            error.valid_up_to()
                        ),
                    )),
                }
            }
            Err(error) => Err(error),
        }
    }
}
