pub struct Source {
    chars: Vec<char>,
    position: usize,
}

impl Source {
    fn new(string: &str) -> Self {
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
mod test {
    use super::*;

    #[test]
    fn consume_returns_none_for_empty() {
        assert_eq!(None, Source::new("").consume());
    }

    #[test]
    fn consume_processes_chars() {
        let mut source = Source::new("abc");
        assert_eq!(Some('a'), source.consume());
        assert_eq!(Some('b'), source.consume());
        assert_eq!(Some('c'), source.consume());
        assert_eq!(None, source.peek());
    }

    #[test]
    fn peek_returns_none_for_empty() {
        assert_eq!(None, Source::new("").peek());
    }

    #[test]
    fn peek_returns_chars_at_positions() {
        let mut source = Source::new("abc");
        assert_eq!(Some('a'), source.peek());
        source.consume();
        assert_eq!(Some('b'), source.peek());
        source.consume();
        assert_eq!(Some('c'), source.peek());
        source.consume();
        assert_eq!(None, source.peek());
    }

    #[test]
    fn position_starts_at_zero() {
        assert_eq!(0, Source::new("").position());
        assert_eq!(0, Source::new("abc").position());
    }

    #[test]
    fn position_is_unchanged_by_peek() {
        let source = Source::new("abc");
        source.peek();
        assert_eq!(0, source.position());
    }

    #[test]
    fn position_is_incremented_by_consume() {
        let mut source = Source::new("abc");
        assert_eq!(0, source.position());
        source.consume();
        assert_eq!(1, source.position());
        source.consume();
        assert_eq!(2, source.position());
        source.consume();
        assert_eq!(3, source.position());
    }
}
