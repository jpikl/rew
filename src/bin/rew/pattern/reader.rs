use crate::pattern::char::{AsChar, Chars};

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

    #[test]
    fn position() {
        let mut reader = make_reader();
        assert_eq!(reader.position(), 0);

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
    fn end() {
        assert_eq!(make_reader().end(), 5);
    }

    #[test]
    fn seek() {
        let mut reader = make_reader();

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
    fn seek_to_end() {
        let mut reader = make_reader();

        reader.seek_to_end();
        assert_eq!(reader.position(), 5);

        reader.seek_to_end();
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn peek() {
        let mut reader = make_reader();

        reader.seek_to(0);
        assert_eq!(reader.peek(), Some(&Char::Raw('a')));
        assert_eq!(reader.position(), 0);

        reader.seek_to(1);
        assert_eq!(reader.peek(), Some(&Char::Escaped('b', ['x', 'y'])));
        assert_eq!(reader.position(), 1);

        reader.seek_to(2);
        assert_eq!(reader.peek(), Some(&Char::Raw('č')));
        assert_eq!(reader.position(), 3);

        reader.seek_to(3);
        assert_eq!(reader.peek(), None);
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn peek_char() {
        let mut reader = make_reader();

        reader.seek_to(0);
        assert_eq!(reader.peek_char(), Some('a'));
        assert_eq!(reader.position(), 0);

        reader.seek_to(1);
        assert_eq!(reader.peek_char(), Some('b'));
        assert_eq!(reader.position(), 1);

        reader.seek_to(2);
        assert_eq!(reader.peek_char(), Some('č'));
        assert_eq!(reader.position(), 3);

        reader.seek_to(3);
        assert_eq!(reader.peek_char(), None);
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn peek_to_end() {
        let mut reader = make_reader();

        reader.seek_to(0);
        assert_eq!(
            reader.peek_to_end(),
            Chars::from(
                &[
                    Char::Raw('a'),
                    Char::Escaped('b', ['x', 'y']),
                    Char::Raw('č')
                ][..]
            )
        );
        assert_eq!(reader.position(), 0);

        reader.seek_to(1);
        assert_eq!(
            reader.peek_to_end(),
            Chars::from(&[Char::Escaped('b', ['x', 'y']), Char::Raw('č')][..])
        );
        assert_eq!(reader.position(), 1);

        reader.seek_to(2);
        assert_eq!(reader.peek_to_end(), Chars::from(&[Char::Raw('č')][..]));
        assert_eq!(reader.position(), 3);

        reader.seek_to(3);
        assert_eq!(reader.peek_to_end(), Chars::from(&[][..]));
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn read() {
        let mut reader = make_reader();

        assert_eq!(reader.read(), Some(&Char::Raw('a')));
        assert_eq!(reader.position(), 1);

        assert_eq!(reader.read(), Some(&Char::Escaped('b', ['x', 'y'])));
        assert_eq!(reader.position(), 3);

        assert_eq!(reader.read(), Some(&Char::Raw('č')));
        assert_eq!(reader.position(), 5);

        assert_eq!(reader.read(), None);
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn read_char() {
        let mut reader = make_reader();

        assert_eq!(reader.read_char(), Some('a'));
        assert_eq!(reader.position(), 1);

        assert_eq!(reader.read_char(), Some('b'));
        assert_eq!(reader.position(), 3);

        assert_eq!(reader.read_char(), Some('č'));
        assert_eq!(reader.position(), 5);

        assert_eq!(reader.read_char(), None);
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn read_to_end() {
        let mut reader = make_reader();

        reader.seek_to(0);
        assert_eq!(
            reader.read_to_end(),
            Chars::from(
                &[
                    Char::Raw('a'),
                    Char::Escaped('b', ['x', 'y']),
                    Char::Raw('č')
                ][..]
            )
        );
        assert_eq!(reader.position(), 5);

        reader.seek_to(1);
        assert_eq!(
            reader.read_to_end(),
            Chars::from(&[Char::Escaped('b', ['x', 'y']), Char::Raw('č')][..])
        );
        assert_eq!(reader.position(), 5);

        reader.seek_to(2);
        assert_eq!(reader.read_to_end(), Chars::from(&[Char::Raw('č')][..]));
        assert_eq!(reader.position(), 5);

        reader.seek_to(3);
        assert_eq!(reader.read_to_end(), Chars::from(&[][..]));
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn read_until() {
        let mut reader = make_reader();

        reader.seek_to(0);
        assert_eq!(reader.read_until(&Char::Raw('a')), Chars::from(&[][..]));

        reader.seek_to(0);
        assert_eq!(
            reader.read_until(&Char::Raw('b')),
            Chars::from(&make_chars()[..1])
        );

        reader.seek_to(0);
        assert_eq!(
            reader.read_until(&Char::Raw('č')),
            Chars::from(&make_chars()[..2])
        );

        reader.seek_to(0);
        assert_eq!(
            reader.read_until(&Char::Raw('?')),
            Chars::from(&make_chars()[..])
        );
    }

    fn make_reader() -> Reader<Char> {
        Reader::new(make_chars())
    }

    fn make_chars() -> Vec<Char> {
        vec![
            Char::Raw('a'),
            Char::Escaped('b', ['x', 'y']),
            Char::Raw('č'),
        ]
    }
}
