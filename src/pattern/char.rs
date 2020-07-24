pub type EscapeSequence = [char; 2];

#[derive(Debug, PartialEq, Clone)]
pub enum Char {
    Raw(char),
    Escaped(char, EscapeSequence),
}

impl Char {
    pub fn join(chars: &[Char]) -> String {
        chars.iter().map(Char::as_char).collect()
    }
}

impl From<char> for Char {
    fn from(value: char) -> Self {
        Char::Raw(value)
    }
}

pub trait AsChar: From<char> {
    fn as_char(&self) -> char;

    fn len_utf8(&self) -> usize;
}

impl AsChar for char {
    fn as_char(&self) -> char {
        *self
    }

    fn len_utf8(&self) -> usize {
        char::len_utf8(*self)
    }
}

impl AsChar for Char {
    fn as_char(&self) -> char {
        match self {
            Self::Raw(value) => *value,
            Self::Escaped(value, _) => *value,
        }
    }

    fn len_utf8(&self) -> usize {
        match self {
            Self::Raw(value) => value.len_utf8(),
            Self::Escaped(_, sequence) => sequence[0].len_utf8() + sequence[1].len_utf8(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_from_char() {
        assert_eq!(Char::from('a'), Char::Raw('a'));
    }

    #[test]
    fn raw_as_char() {
        assert_eq!(Char::Raw('a').as_char(), 'a');
    }

    #[test]
    fn raw_len_utf8() {
        assert_eq!(Char::Raw('a').len_utf8(), 1);
        assert_eq!(Char::Raw('á').len_utf8(), 2);
    }

    #[test]
    fn escaped_as_char() {
        assert_eq!(Char::Escaped('a', ['b', 'c']).as_char(), 'a');
    }

    #[test]
    fn escaped_len_utf8() {
        assert_eq!(Char::Escaped('a', ['b', 'c']).len_utf8(), 2);
        assert_eq!(Char::Escaped('a', ['á', 'č']).len_utf8(), 4);
    }

    #[test]
    fn join() {
        let chars = [Char::Raw('a'), Char::Escaped('b', ['c', 'd'])];
        assert_eq!(Char::join(&chars), String::from("ab"));
    }
}
