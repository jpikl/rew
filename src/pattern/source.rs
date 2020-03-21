pub struct Source {
    chars: Vec<char>,
    position: usize,
}

impl Source {
    pub fn new(string: &str) -> Self {
        Self {
            chars: string.chars().collect(),
            position: 0,
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn consume(&mut self) -> Option<char> {
        self.peek().map(|ch| {
            self.position += 1;
            ch
        })
    }

    pub fn peek(&self) -> Option<char> {
        if self.position < self.chars.len() {
            Some(self.chars[self.position])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_returns_none_for_empty() {
        assert_eq!(Source::new("").consume(), None);
    }

    #[test]
    fn consume_processes_chars() {
        let mut source = Source::new("abc");
        assert_eq!(source.consume(), Some('a'));
        assert_eq!(source.consume(), Some('b'));
        assert_eq!(source.consume(), Some('c'));
        assert_eq!(source.peek(), None);
    }

    #[test]
    fn peek_returns_none_for_empty() {
        assert_eq!(Source::new("").peek(), None);
    }

    #[test]
    fn peek_returns_chars_at_positions() {
        let mut source = Source::new("abc");
        assert_eq!(source.peek(), Some('a'));
        source.consume();
        assert_eq!(source.peek(), Some('b'));
        source.consume();
        assert_eq!(source.peek(), Some('c'));
        source.consume();
        assert_eq!(source.peek(), None);
    }

    #[test]
    fn position_starts_at_zero() {
        assert_eq!(Source::new("").position(), 0);
        assert_eq!(Source::new("abc").position(), 0);
    }

    #[test]
    fn position_is_unchanged_by_peek() {
        let source = Source::new("abc");
        source.peek();
        assert_eq!(source.position(), 0);
    }

    #[test]
    fn position_is_incremented_by_consume() {
        let mut source = Source::new("abc");
        assert_eq!(source.position(), 0);
        source.consume();
        assert_eq!(source.position(), 1);
        source.consume();
        assert_eq!(source.position(), 2);
        source.consume();
        assert_eq!(source.position(), 3);
    }
}
