pub struct Reader {
    chars: Vec<char>,
    position: usize,
}

impl Reader {
    pub fn new(string: &str) -> Self {
        Self {
            chars: string.chars().collect(),
            position: 0,
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn end(&self) -> usize {
        self.chars.len()
    }

    pub fn read(&mut self) -> Option<char> {
        self.peek().map(|ch| {
            self.position += 1;
            ch
        })
    }

    pub fn peek(&self) -> Option<char> {
        if self.position < self.end() {
            Some(self.chars[self.position])
        } else {
            None
        }
    }

    pub fn consume(&mut self) -> &[char] {
        let remainder = &self.chars[self.position..];
        self.position = self.end();
        remainder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_returns_none_for_empty() {
        assert_eq!(Reader::new("").read(), None);
    }

    #[test]
    fn readt_consumes_chars() {
        let mut reader = Reader::new("abc");
        assert_eq!(reader.read(), Some('a'));
        assert_eq!(reader.read(), Some('b'));
        assert_eq!(reader.read(), Some('c'));
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn peek_returns_none_for_empty() {
        assert_eq!(Reader::new("").peek(), None);
    }

    #[test]
    fn peek_returns_chars_at_positions() {
        let mut reader = Reader::new("abc");
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
        let mut reader = Reader::new("abc");
        reader.read();
        assert_eq!(reader.consume(), &['b', 'c']);
        assert_eq!(reader.consume(), &[] as &[char]);
    }

    #[test]
    fn position_starts_at_zero() {
        assert_eq!(Reader::new("").position(), 0);
        assert_eq!(Reader::new("abc").position(), 0);
    }

    #[test]
    fn position_is_unchanged_by_peek() {
        let reader = Reader::new("abc");
        reader.peek();
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn position_is_incremented_by_read() {
        let mut reader = Reader::new("abc");
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
        let mut reader = Reader::new("abc");
        assert_eq!(reader.position(), 0);
        reader.consume();
        assert_eq!(reader.position(), 3);
    }
}
