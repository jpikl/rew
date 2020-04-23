use crate::pattern::char::Char;

pub struct Reader {
    chars: Vec<Char>,
    index: usize,
}

impl From<&str> for Reader {
    fn from(string: &str) -> Self {
        Self::new(Char::raw_vec(string))
    }
}

// TODO char vs Char - trait template + default implementations
// TODO use Chars for iterating string
// TODO remove *_value methods
impl Reader {
    pub fn new(chars: Vec<Char>) -> Self {
        Self { chars, index: 0 }
    }

    pub fn position(&self) -> usize {
        Char::sum_len(&self.chars[..self.index])
    }

    pub fn end(&self) -> usize {
        Char::sum_len(&self.chars)
    }

    pub fn seek(&mut self) {
        self.seek_to(self.index + 1)
    }

    pub fn seek_to_end(&mut self) {
        self.seek_to(self.chars.len())
    }

    fn seek_to(&mut self, index: usize) {
        self.index = self.chars.len().min(index);
    }

    pub fn peek(&self) -> Option<&Char> {
        self.peek_at(self.index)
    }

    fn peek_at(&self, index: usize) -> Option<&Char> {
        if index >= self.chars.len() {
            None
        } else {
            Some(&self.chars[index])
        }
    }

    pub fn peek_value(&self) -> Option<char> {
        self.peek().map(Char::value)
    }

    pub fn peek_to_end(&self) -> &[Char] {
        self.peek_to_end_at(self.index)
    }

    fn peek_to_end_at(&self, index: usize) -> &[Char] {
        &self.chars[index..]
    }

    pub fn read(&mut self) -> Option<&Char> {
        let index = self.index;
        self.seek();
        self.peek_at(index)
    }

    pub fn read_value(&mut self) -> Option<char> {
        self.read().map(Char::value)
    }

    pub fn read_to_end(&mut self) -> &[Char] {
        let index = self.index;
        self.seek_to_end();
        self.peek_to_end_at(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_starts_at_zero() {
        assert_eq!(make_empty_reader().position(), 0);
        assert_eq!(make_reader().position(), 0);
    }

    #[test]
    fn position_returns_values_at_indices() {
        let mut reader = make_reader();
        reader.seek_to(0);
        assert_eq!(reader.position(), 0);
        reader.seek_to(1);
        assert_eq!(reader.position(), 1);
        reader.seek_to(2);
        assert_eq!(reader.position(), 3);
        reader.seek_to(3);
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn end_returns_last_position() {
        assert_eq!(make_empty_reader().end(), 0);
        assert_eq!(make_reader().end(), 5);
    }

    #[test]
    fn seek_increments_position() {
        let mut reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.seek();
        assert_eq!(reader.position(), 1);
        reader.seek();
        assert_eq!(reader.position(), 3);
        reader.seek();
        assert_eq!(reader.position(), 5);
        reader.seek();
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn seek_to_end_moves_position_to_end() {
        let mut reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.seek_to_end();
        assert_eq!(reader.position(), 5);
        reader.seek_to_end();
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn peek_returns_none_for_empty() {
        assert_eq!(make_empty_reader().peek(), None);
    }

    #[test]
    fn peek_returns_chars_at_indices() {
        let mut reader = make_reader();
        reader.seek_to(0);
        assert_eq!(reader.peek(), Some(&Char::Raw('a')));
        reader.seek_to(1);
        assert_eq!(reader.peek(), Some(&Char::Escaped('b', ['x', 'y'])));
        reader.seek_to(2);
        assert_eq!(reader.peek(), Some(&Char::Raw('č')));
        reader.seek_to(3);
        assert_eq!(reader.peek(), None);
    }

    #[test]
    fn peek_does_not_change_position() {
        let reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.peek();
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn peek_value_returns_none_for_empty() {
        assert_eq!(make_empty_reader().peek_value(), None);
    }

    #[test]
    fn peek_value_returns_char_values_at_indices() {
        let mut reader = make_reader();
        reader.seek_to(0);
        assert_eq!(reader.peek_value(), Some('a'));
        reader.seek_to(1);
        assert_eq!(reader.peek_value(), Some('b'));
        reader.seek_to(2);
        assert_eq!(reader.peek_value(), Some('č'));
        reader.seek_to(3);
        assert_eq!(reader.peek_value(), None);
    }

    #[test]
    fn peek_value_does_not_change_position() {
        let reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.peek_value();
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn peek_to_end_returns_remaining_chars() {
        let mut reader = make_reader();
        reader.seek_to(0);
        assert_eq!(
            reader.peek_to_end(),
            [
                Char::Raw('a'),
                Char::Escaped('b', ['x', 'y']),
                Char::Raw('č')
            ]
        );
        reader.seek_to(1);
        assert_eq!(
            reader.peek_to_end(),
            [Char::Escaped('b', ['x', 'y']), Char::Raw('č')]
        );
        reader.seek_to(2);
        assert_eq!(reader.peek_to_end(), [Char::Raw('č')]);
        reader.seek_to(3);
        assert_eq!(reader.peek_to_end(), []);
    }

    #[test]
    fn peek_to_end_does_not_change_position() {
        let reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.peek_to_end();
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn read_returns_none_for_empty() {
        assert_eq!(make_empty_reader().read(), None);
    }

    #[test]
    fn read_consumes_chars() {
        let mut reader = make_reader();
        assert_eq!(reader.read(), Some(&Char::Raw('a')));
        assert_eq!(reader.read(), Some(&Char::Escaped('b', ['x', 'y'])));
        assert_eq!(reader.read(), Some(&Char::Raw('č')));
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn read_increments_position() {
        let mut reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.read();
        assert_eq!(reader.position(), 1);
        reader.read();
        assert_eq!(reader.position(), 3);
        reader.read();
        assert_eq!(reader.position(), 5);
        reader.read();
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn read_value_returns_none_for_empty() {
        assert_eq!(make_empty_reader().read_value(), None);
    }

    #[test]
    fn read_value_consumes_char_values() {
        let mut reader = make_reader();
        assert_eq!(reader.read_value(), Some('a'));
        assert_eq!(reader.read_value(), Some('b'));
        assert_eq!(reader.read_value(), Some('č'));
        assert_eq!(reader.read_value(), None);
    }

    #[test]
    fn read_value_increments_position() {
        let mut reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.read_value();
        assert_eq!(reader.position(), 1);
        reader.read_value();
        assert_eq!(reader.position(), 3);
        reader.read_value();
        assert_eq!(reader.position(), 5);
        reader.read_value();
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn read_to_end_returns_remaining_chars() {
        let mut reader = make_reader();
        reader.seek_to(0);
        assert_eq!(
            reader.read_to_end(),
            [
                Char::Raw('a'),
                Char::Escaped('b', ['x', 'y']),
                Char::Raw('č')
            ]
        );
        assert_eq!(reader.read_to_end(), []);
        reader.seek_to(1);
        assert_eq!(
            reader.read_to_end(),
            [Char::Escaped('b', ['x', 'y']), Char::Raw('č')]
        );
        assert_eq!(reader.read_to_end(), []);
        reader.seek_to(2);
        assert_eq!(reader.read_to_end(), [Char::Raw('č')]);
        assert_eq!(reader.read_to_end(), []);
        reader.seek_to(3);
        assert_eq!(reader.read_to_end(), []);
    }

    #[test]
    fn read_to_end_moves_position_to_end() {
        let mut reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.read_to_end();
        assert_eq!(reader.position(), 5);
        reader.read_to_end();
        assert_eq!(reader.position(), 5);
    }

    fn make_empty_reader() -> Reader {
        Reader::new(Vec::new())
    }

    fn make_reader() -> Reader {
        Reader::new(vec![
            Char::Raw('a'),
            Char::Escaped('b', ['x', 'y']),
            Char::Raw('č'),
        ])
    }
}
