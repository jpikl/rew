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

    pub fn peek(&self) -> Option<char> {
        if self.position < self.chars.len() {
            Some(self.chars[self.position])
        } else {
            None
        }
    }
}

impl Iterator for Reader {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.peek().map(|ch| {
            self.position += 1;
            ch
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_returns_none_for_empty() {
        assert_eq!(Reader::new("").next(), None);
    }

    #[test]
    fn next_consumes_chars() {
        let mut reader = Reader::new("abc");
        assert_eq!(reader.next(), Some('a'));
        assert_eq!(reader.next(), Some('b'));
        assert_eq!(reader.next(), Some('c'));
        assert_eq!(reader.peek(), None);
    }

    #[test]
    fn peek_returns_none_for_empty() {
        assert_eq!(Reader::new("").peek(), None);
    }

    #[test]
    fn peek_returns_chars_at_positions() {
        let mut reader = Reader::new("abc");
        assert_eq!(reader.peek(), Some('a'));
        reader.next();
        assert_eq!(reader.peek(), Some('b'));
        reader.next();
        assert_eq!(reader.peek(), Some('c'));
        reader.next();
        assert_eq!(reader.peek(), None);
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
    fn position_is_incremented_by_next() {
        let mut reader = Reader::new("abc");
        assert_eq!(reader.position(), 0);
        reader.next();
        assert_eq!(reader.position(), 1);
        reader.next();
        assert_eq!(reader.position(), 2);
        reader.next();
        assert_eq!(reader.position(), 3);
    }
}
