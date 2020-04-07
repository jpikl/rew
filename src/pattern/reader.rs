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

    pub fn is_consumed(&self) -> bool {
        self.index >= self.chars.len()
    }

    pub fn seek(&mut self, index: usize) -> usize {
        let prev_index = self.index;
        self.index = index;
        prev_index
    }

    pub fn read(&mut self) -> Option<&Char> {
        if self.is_consumed() {
            None
        } else {
            let prev_index = self.seek(self.index + 1);
            Some(&self.chars[prev_index])
        }
    }

    pub fn read_value(&mut self) -> Option<char> {
        self.read().map(Char::value)
    }

    pub fn read_to_end(&mut self) -> &[Char] {
        let prev_index = self.seek(self.chars.len());
        &self.chars[prev_index..]
    }

    pub fn peek(&self) -> Option<&Char> {
        if self.is_consumed() {
            None
        } else {
            Some(&self.chars[self.index])
        }
    }

    pub fn peek_value(&self) -> Option<char> {
        self.peek().map(Char::value)
    }

    pub fn peek_to_end(&self) -> &[Char] {
        &self.chars[self.index..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_returns_none_for_empty() {
        assert_eq!(make_empty_reader().read(), None);
    }

    #[test]
    fn read_consumes_chars() {
        let mut reader = make_reader();
        assert_eq!(reader.read(), Some(&Char::Raw('a')));
        assert_eq!(reader.read(), Some(&Char::Escaped('\\', 'b')));
        assert_eq!(reader.read(), Some(&Char::Raw('c')));
        assert_eq!(reader.read(), None);
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
        assert_eq!(reader.read_value(), Some('c'));
        assert_eq!(reader.read_value(), None);
    }

    #[test]
    fn read_to_end_returns_remaining_chars() {
        let mut reader = make_reader();
        reader.seek(1);
        assert_eq!(
            reader.read_to_end(),
            [Char::Escaped('\\', 'b'), Char::Raw('c')]
        );
        assert_eq!(reader.read_to_end(), &[] as &[Char]);
    }

    #[test]
    fn peek_returns_none_for_empty() {
        assert_eq!(Reader::from("").peek(), None);
    }

    #[test]
    fn peek_returns_chars_at_indices() {
        let mut reader = make_reader();
        assert_eq!(reader.peek(), Some(&Char::Raw('a')));
        reader.seek(1);
        assert_eq!(reader.peek(), Some(&Char::Escaped('\\', 'b')));
        reader.seek(2);
        assert_eq!(reader.peek(), Some(&Char::Raw('c')));
        reader.seek(3);
        assert_eq!(reader.peek(), None);
    }

    #[test]
    fn peek_value_returns_none_for_empty() {
        assert_eq!(Reader::from("").peek_value(), None);
    }

    #[test]
    fn peek_value_returns_char_values_at_indices() {
        let mut reader = make_reader();
        assert_eq!(reader.peek_value(), Some('a'));
        reader.seek(1);
        assert_eq!(reader.peek_value(), Some('b'));
        reader.seek(2);
        assert_eq!(reader.peek_value(), Some('c'));
        reader.seek(3);
        assert_eq!(reader.peek_value(), None);
    }

    #[test]
    fn peek_to_end_returns_remaining_chars() {
        let mut reader = make_reader();
        reader.seek(1);
        assert_eq!(
            reader.peek_to_end(),
            [Char::Escaped('\\', 'b'), Char::Raw('c')]
        );
        assert_eq!(
            reader.peek_to_end(),
            [Char::Escaped('\\', 'b'), Char::Raw('c')]
        );
    }

    fn is_consumed_returns_true_for_empty() {
        assert_eq!(make_empty_reader().is_consumed(), true);
    }

    fn is_consumed_returns_true_when_consumed() {
        let mut reader = make_reader();
        assert_eq!(reader.is_consumed(), false);
        reader.seek(1);
        assert_eq!(reader.is_consumed(), false);
        reader.seek(2);
        assert_eq!(reader.is_consumed(), false);
        reader.seek(3);
        assert_eq!(reader.is_consumed(), true);
    }

    #[test]
    fn position_starts_at_zero() {
        assert_eq!(make_empty_reader().position(), 0);
        assert_eq!(make_reader().position(), 0);
    }

    #[test]
    fn position_is_changed_by_seek() {
        let mut reader = make_reader();
        reader.seek(0);
        assert_eq!(reader.position(), 0);
        reader.seek(1);
        assert_eq!(reader.position(), 1);
        reader.seek(2);
        assert_eq!(reader.position(), 3);
        reader.seek(3);
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn position_is_unchanged_by_peek() {
        let reader = make_reader();
        reader.peek_value();
        assert_eq!(reader.position(), 0);
    }

    #[test]
    fn position_is_increased_by_read() {
        let mut reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.read();
        assert_eq!(reader.position(), 1);
        reader.read();
        assert_eq!(reader.position(), 3);
        reader.read();
        assert_eq!(reader.position(), 4);
    }

    #[test]
    fn position_is_moved_to_the_end_by_read_to_end() {
        let mut reader = make_reader();
        assert_eq!(reader.position(), 0);
        reader.read_to_end();
        assert_eq!(reader.position(), 4);
    }

    fn make_empty_reader() -> Reader {
        Reader::new(Vec::new())
    }

    fn make_reader() -> Reader {
        Reader::new(vec![
            Char::Raw('a'),
            Char::Escaped('\\', 'b'),
            Char::Raw('c'),
        ])
    }
}
