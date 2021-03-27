use crate::pattern::char::{AsChar, Chars};

pub struct Reader<T: AsChar> {
    chars: Vec<T>,
    index: usize,
}

impl<T: AsChar> From<&str> for Reader<T> {
    fn from(input: &str) -> Self {
        Self::new(input.chars().map(T::from).collect())
    }
}

impl<T: AsChar> Reader<T> {
    pub fn new(chars: Vec<T>) -> Self {
        Self { chars, index: 0 }
    }

    pub fn position(&self) -> usize {
        Chars::from(&self.chars[..self.index]).len_utf8()
    }

    pub fn end(&self) -> usize {
        Chars::from(&self.chars[..]).len_utf8()
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
        if index < self.chars.len() {
            Some(&self.chars[index])
        } else {
            None
        }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.peek().map(T::as_char)
    }

    pub fn peek_to_end(&self) -> Chars<T> {
        self.peek_to_end_at(self.index)
    }

    fn peek_to_end_at(&self, index: usize) -> Chars<T> {
        Chars::from(&self.chars[index..])
    }

    pub fn read(&mut self) -> Option<&T> {
        let index = self.index;
        self.seek();
        self.peek_at(index)
    }

    pub fn read_char(&mut self) -> Option<char> {
        self.read().map(T::as_char)
    }

    pub fn read_expected(&mut self, expected: char) -> bool {
        match self.peek_char() {
            Some(value) if value == expected => {
                self.seek();
                true
            }
            _ => false,
        }
    }

    pub fn read_to_end(&mut self) -> Chars<T> {
        let index = self.index;
        self.seek_to_end();
        self.peek_to_end_at(index)
    }

    pub fn read_until(&mut self, delimiter: &T) -> Chars<T> {
        for i in self.index..self.chars.len() {
            if self.chars[i].as_char() == delimiter.as_char() {
                let index = self.index;
                self.seek_to(i + 1);
                return Chars::from(&self.chars[index..i]);
            }
        }
        self.read_to_end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::char::Char;
    use test_case::test_case;

    const CHARS: [Char; 3] = [
        Char::Raw('a'),
        Char::Escaped('b', ['x', 'y']),
        Char::Raw('č'),
    ];

    #[test_case(0, 0; "index 0")]
    #[test_case(1, 1; "index 1")]
    #[test_case(2, 3; "index 2")]
    #[test_case(3, 5; "index 3")]
    fn position(index: usize, position: usize) {
        assert_eq!(make_reader_at(index).position(), position);
    }

    #[test_case(0, 5; "index 0")]
    #[test_case(1, 5; "index 1")]
    #[test_case(2, 5; "index 2")]
    #[test_case(3, 5; "index 3")]
    fn end(index: usize, position: usize) {
        assert_eq!(make_reader_at(index).end(), position);
    }

    #[test_case(0, 1; "index 0")]
    #[test_case(1, 3; "index 1")]
    #[test_case(2, 5; "index 2")]
    #[test_case(3, 5; "index 3")]
    fn seek(index: usize, position: usize) {
        let mut reader = make_reader_at(index);
        reader.seek();
        assert_eq!(reader.position(), position);
    }

    #[test_case(0, 5; "index 0")]
    #[test_case(1, 5; "index 1")]
    #[test_case(2, 5; "index 2")]
    #[test_case(3, 5; "index 3")]
    fn seek_to_end(index: usize, position: usize) {
        let mut reader = make_reader_at(index);
        reader.seek_to_end();
        assert_eq!(reader.position(), position);
    }

    #[test_case(0, Some(&CHARS[0]), 0; "index 0")]
    #[test_case(1, Some(&CHARS[1]), 1; "index 1")]
    #[test_case(2, Some(&CHARS[2]), 3; "index 2")]
    #[test_case(3, None,            5; "index 3")]
    fn peek(index: usize, result: Option<&Char>, position: usize) {
        let reader = make_reader_at(index);
        assert_eq!(reader.peek(), result);
        assert_eq!(reader.position(), position);
    }

    #[test_case(0, Some('a'), 0; "index 0")]
    #[test_case(1, Some('b'), 1; "index 1")]
    #[test_case(2, Some('č'), 3; "index 2")]
    #[test_case(3, None,      5; "index 3")]
    fn peek_char(index: usize, result: Option<char>, position: usize) {
        let reader = make_reader_at(index);
        assert_eq!(reader.peek_char(), result);
        assert_eq!(reader.position(), position);
    }

    #[test_case(0, &CHARS[..],  0; "index 0")]
    #[test_case(1, &CHARS[1..], 1; "index 1")]
    #[test_case(2, &CHARS[2..], 3; "index 2")]
    #[test_case(3, &[][..],     5; "index 3")]
    fn peek_to_end(index: usize, result: &[Char], position: usize) {
        let reader = make_reader_at(index);
        assert_eq!(reader.peek_to_end(), result.into());
        assert_eq!(reader.position(), position);
    }

    #[test_case(0, Some(&CHARS[0]), 1; "index 0")]
    #[test_case(1, Some(&CHARS[1]), 3; "index 1")]
    #[test_case(2, Some(&CHARS[2]), 5; "index 2")]
    #[test_case(3, None,            5; "index 3")]
    fn read(index: usize, result: Option<&Char>, position: usize) {
        let mut reader = make_reader_at(index);
        assert_eq!(reader.read(), result);
        assert_eq!(reader.position(), position);
    }

    #[test_case(0, Some('a'), 1; "index 0")]
    #[test_case(1, Some('b'), 3; "index 1")]
    #[test_case(2, Some('č'), 5; "index 2")]
    #[test_case(3, None,      5; "index 3")]
    fn read_char(index: usize, result: Option<char>, position: usize) {
        let mut reader = make_reader_at(index);
        assert_eq!(reader.read_char(), result);
        assert_eq!(reader.position(), position);
    }

    #[test_case(0, 'a', true,  1; "index 0 hit")]
    #[test_case(1, 'b', true,  3; "index 1 hit")]
    #[test_case(2, 'č', true,  5; "index 2 hit")]
    #[test_case(0, 'x', false, 0; "index 0 miss")]
    #[test_case(1, 'x', false, 1; "index 1 miss")]
    #[test_case(2, 'x', false, 3; "index 2 miss")]
    #[test_case(3, 'x', false, 5; "index 3 miss")]
    fn read_expected(index: usize, expected: char, result: bool, position: usize) {
        let mut reader = make_reader_at(index);
        assert_eq!(reader.read_expected(expected), result);
        assert_eq!(reader.position(), position);
    }

    #[test_case(0, &CHARS[..],  5; "index 0")]
    #[test_case(1, &CHARS[1..], 5; "index 1")]
    #[test_case(2, &CHARS[2..], 5; "index 2")]
    #[test_case(3, &[][..],     5; "index 3")]
    fn read_to_end(index: usize, result: &[Char], position: usize) {
        let mut reader = make_reader_at(index);
        assert_eq!(reader.read_to_end(), result.into());
        assert_eq!(reader.position(), position);
    }

    #[test_case(0, &CHARS[0],       &[][..],      1; "index 0 to 0")]
    #[test_case(1, &CHARS[1],       &[][..],      3; "index 1 to 1")]
    #[test_case(2, &CHARS[2],       &[][..],      5; "index 2 to 2")]
    #[test_case(0, &CHARS[1],       &CHARS[..1],  3; "index 0 to 1")]
    #[test_case(0, &CHARS[2],       &CHARS[..2],  5; "index 0 to 2")]
    #[test_case(1, &CHARS[2],       &CHARS[1..2], 5; "index 1 to 2")]
    #[test_case(1, &Char::Raw('x'), &CHARS[1..],  5; "index 1 to end")]
    #[test_case(0, &Char::Raw('x'), &CHARS[..],   5; "index 0 to end")]
    #[test_case(2, &Char::Raw('x'), &CHARS[2..],  5; "index 2 to end")]
    #[test_case(3, &Char::Raw('x'), &[][..],      5; "index 3 to end")]
    fn read_until(index: usize, delimiter: &Char, result: &[Char], position: usize) {
        let mut reader = make_reader_at(index);
        assert_eq!(reader.read_until(delimiter), result.into());
        assert_eq!(reader.position(), position);
    }

    fn make_reader_at(index: usize) -> Reader<Char> {
        let mut reader = Reader::new(CHARS.into());
        if index > 0 {
            reader.seek_to(index)
        }
        reader
    }
}
