use crate::pattern::char::Char;

pub struct Reader {
    chars: Vec<Char>,
    position: usize,
}

impl From<&str> for Reader {
    fn from(string: &str) -> Self {
        Self::new(Char::raw_vec(string))
    }
}

impl Reader {
    pub fn new(chars: Vec<Char>) -> Self {
        Self { chars, position: 0 }
    }

    pub fn position(&self) -> usize {
        Char::sum_len(&self.chars[..self.position])
    }

    pub fn end(&self) -> usize {
        Char::sum_len(&self.chars)
    }

    pub fn read(&mut self) -> Option<char> {
        self.peek().map(|ch| {
            self.position += 1;
            ch
        })
    }

    pub fn peek(&self) -> Option<char> {
        if self.position < self.end() {
            Some(self.chars[self.position].value())
        } else {
            None
        }
    }

    pub fn consume(&mut self) -> String {
        let remainder = &self.chars[self.position..];
        self.position = self.end();
        Char::join(remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_returns_none_for_empty() {
        assert_eq!(Reader::from("").read(), None);
    }

    #[test]
    fn readt_consumes_chars() {
        let mut reader = Reader::from("abc");
        assert_eq!(reader.read(), Some('a'));
        assert_eq!(reader.read(), Some('b'));
        assert_eq!(reader.read(), Some('c'));
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn peek_returns_none_for_empty() {
        assert_eq!(Reader::from("").peek(), None);
    }

    #[test]
    fn peek_returns_chars_at_positions() {
        let mut reader = Reader::from("abc");
        assert_eq!(reader.peek(), Some('a'));
        reader.read();
        assert_eq!(reader.peek(), Some('b'));
        reader.read();
        assert_eq!(reader.peek(), Some('c'));
        reader.read();
        assert_eq!(reader.peek(), None);
    }

    #[test]
    fn consume_returns_remaining_chars() {
        let mut reader = Reader::from("abc");
        reader.read();
        assert_eq!(reader.consume(), "bc");
        assert_eq!(reader.consume(), "");
    }

    #[test]
    fn position_starts_at_zero() {
        assert_eq!(Reader::from("").position(), 0);
        assert_eq!(Reader::from("abc").position(), 0);
    }

    #[test]
    fn position_is_unchanged_by_peek() {
        let reader = Reader::from("abc");
        reader.peek();
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn position_is_incremented_by_read() {
        let mut reader = Reader::from("abc");
        assert_eq!(reader.position(), 0);
        reader.read();
        assert_eq!(reader.position(), 1);
        reader.read();
        assert_eq!(reader.position(), 2);
        reader.read();
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn position_is_moved_to_the_end_by_consume() {
        let mut reader = Reader::from("abc");
        assert_eq!(reader.position(), 0);
        reader.consume();
        assert_eq!(reader.position(), 3);
    }
}
