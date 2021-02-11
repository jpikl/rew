use std::ops::Deref;

pub type EscapeSequence = [char; 2];

#[derive(Debug, PartialEq, Clone)]
pub enum Char {
    Raw(char),
    Escaped(char, EscapeSequence),
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

#[derive(Debug, PartialEq)]
pub struct Chars<'a, T: AsChar>(pub &'a [T]);

impl<'a, T: AsChar> Deref for Chars<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: AsChar> Chars<'a, T> {
    pub fn len_utf8(&self) -> usize {
        self.0.iter().fold(0, |sum, char| sum + char.len_utf8())
    }
}

impl<'a, T: AsChar> ToString for Chars<'a, T> {
    fn to_string(&self) -> String {
        self.0.iter().map(T::as_char).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod raw {
        use super::*;

        #[test]
        fn from_char() {
            assert_eq!(Char::from('a'), Char::Raw('a'));
        }

        #[test]
        fn as_char() {
            assert_eq!(Char::Raw('a').as_char(), 'a');
        }

        #[test]
        fn len_utf8() {
            assert_eq!(Char::Raw('a').len_utf8(), 1);
            assert_eq!(Char::Raw('á').len_utf8(), 2);
        }
    }

    mod escaped {
        use super::*;

        #[test]
        fn as_char() {
            assert_eq!(Char::Escaped('a', ['b', 'c']).as_char(), 'a');
        }

        #[test]
        fn len_utf8() {
            assert_eq!(Char::Escaped('a', ['b', 'c']).len_utf8(), 2);
            assert_eq!(Char::Escaped('a', ['á', 'č']).len_utf8(), 4);
        }
    }

    mod chars {
        use super::*;

        #[test]
        fn len_utf8() {
            let chars = [Char::Raw('a'), Char::Escaped('b', ['c', 'd'])];
            assert_eq!(Chars(&chars).len_utf8(), 3);
        }

        #[test]
        fn to_string() {
            let chars = [Char::Raw('a'), Char::Escaped('b', ['c', 'd'])];
            assert_eq!(Chars(&chars).to_string(), String::from("ab"));
        }
    }
}
