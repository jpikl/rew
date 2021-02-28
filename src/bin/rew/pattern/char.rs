use std::fmt;
use std::fmt::Debug;
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

impl fmt::Display for Char {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Raw(value) => fmt_char(*value, formatter),
            Self::Escaped(value, sequence) => {
                fmt_char(*value, formatter)?;
                write!(
                    formatter,
                    " (escape sequence '{}{}')",
                    sequence[0], sequence[1]
                )
            }
        }
    }
}

fn fmt_char(value: char, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "'")?;
    match value {
        '\0' => write!(formatter, "Null"),
        '\n' => write!(formatter, "Line feed"),
        '\r' => write!(formatter, "Carriage return"),
        '\t' => write!(formatter, "Horizontal tab"),
        value => write!(formatter, "{}", value),
    }?;
    write!(formatter, "'")
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
pub struct Chars<'a, T: AsChar>(&'a [T]);

impl<'a, T: AsChar> Chars<'a, T> {
    pub fn len_utf8(&self) -> usize {
        self.0.iter().fold(0, |sum, char| sum + char.len_utf8())
    }
}

impl<'a, T: AsChar> From<&'a [T]> for Chars<'a, T> {
    fn from(chars: &'a [T]) -> Self {
        Self(chars)
    }
}

impl<'a, T: AsChar> Deref for Chars<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
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

    mod char_raw {
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

        #[test]
        fn display() {
            assert_eq!(Char::Raw('a').to_string(), "'a'");
            assert_eq!(Char::Raw('\0').to_string(), "'Null'");
            assert_eq!(Char::Raw('\n').to_string(), "'Line feed'");
            assert_eq!(Char::Raw('\r').to_string(), "'Carriage return'");
            assert_eq!(Char::Raw('\t').to_string(), "'Horizontal tab'");
        }
    }

    mod char_escaped {
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

        #[test]
        fn display() {
            assert_eq!(
                Char::Escaped('a', ['b', 'c']).to_string(),
                "'a' (escape sequence 'bc')"
            );
            assert_eq!(
                Char::Escaped('\0', ['%', '0']).to_string(),
                "'Null' (escape sequence '%0')"
            );
            assert_eq!(
                Char::Escaped('\n', ['%', 'n']).to_string(),
                "'Line feed' (escape sequence '%n')"
            );
            assert_eq!(
                Char::Escaped('\r', ['%', 'r']).to_string(),
                "'Carriage return' (escape sequence '%r')"
            );
            assert_eq!(
                Char::Escaped('\t', ['%', 't']).to_string(),
                "'Horizontal tab' (escape sequence '%t')"
            );
        }
    }

    mod chars {
        use super::*;

        #[test]
        fn from() {
            let chars = [Char::Raw('a'), Char::Escaped('b', ['c', 'd'])];
            assert_eq!(Chars(&chars), Chars::from(&chars[..]));
        }

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
