use crate::pattern::char::AsChar;

pub struct Reader<T: AsChar> {
    chars: Vec<T>,
    index: usize,
}

impl<T: AsChar> From<&str> for Reader<T> {
    fn from(string: &str) -> Self {
        Self::new(string.chars().map(T::from).collect())
    }
}

impl<T: AsChar> Reader<T> {
    pub fn new(chars: Vec<T>) -> Self {
        Self { chars, index: 0 }
    }

    pub fn position(&self) -> usize {
        sum_len_utf8::<T>(&self.chars[..self.index])
    }

    pub fn end(&self) -> usize {
        sum_len_utf8(&self.chars)
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

    pub fn peek(&self) -> Option<&T> {
        self.peek_at(self.index)
    }

    fn peek_at(&self, index: usize) -> Option<&T> {
        if index >= self.chars.len() {
            None
        } else {
            Some(&self.chars[index])
        }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.peek().map(T::as_char)
    }

    pub fn peek_to_end(&self) -> &[T] {
        self.peek_to_end_at(self.index)
    }

    fn peek_to_end_at(&self, index: usize) -> &[T] {
        &self.chars[index..]
    }

    pub fn read(&mut self) -> Option<&T> {
        let index = self.index;
        self.seek();
        self.peek_at(index)
    }

    pub fn read_char(&mut self) -> Option<char> {
        self.read().map(T::as_char)
    }

    pub fn read_to_end(&mut self) -> &[T] {
        let index = self.index;
        self.seek_to_end();
        self.peek_to_end_at(index)
    }
}

fn sum_len_utf8<T: AsChar>(chars: &[T]) -> usize {
    chars.iter().fold(0, |sum, char| sum + char.len_utf8())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::char::Char;

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
    fn peek_char_returns_none_for_empty() {
        assert_eq!(make_empty_reader().peek_char(), None);
    }

    #[test]
    fn peek_char_returns_char_values_at_indices() {
        let mut reader = make_reader();
        reader.seek_to(0);
        assert_eq!(reader.peek_char(), Some('a'));
        reader.seek_to(1);
        assert_eq!(reader.peek_char(), Some('b'));
        reader.seek_to(2);
        assert_eq!(reader.peek_char(), Some('č'));
        reader.seek_to(3);
        assert_eq!(reader.peek_char(), None);
    }

    #[test]
    fn peek_char_does_not_change_position() {
        let reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.peek_char();
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
    fn read_char_returns_none_for_empty() {
        assert_eq!(make_empty_reader().read_char(), None);
    }

    #[test]
    fn read_char_consumes_char_values() {
        let mut reader = make_reader();
        assert_eq!(reader.read_char(), Some('a'));
        assert_eq!(reader.read_char(), Some('b'));
        assert_eq!(reader.read_char(), Some('č'));
        assert_eq!(reader.read_char(), None);
    }

    #[test]
    fn read_char_increments_position() {
        let mut reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.read_char();
        assert_eq!(reader.position(), 1);
        reader.read_char();
        assert_eq!(reader.position(), 3);
        reader.read_char();
        assert_eq!(reader.position(), 5);
        reader.read_char();
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

    fn make_empty_reader() -> Reader<Char> {
        Reader::new(Vec::new())
    }

    fn make_reader() -> Reader<Char> {
        Reader::new(vec![
            Char::Raw('a'),
            Char::Escaped('b', ['x', 'y']),
            Char::Raw('č'),
        ])
    }
}
