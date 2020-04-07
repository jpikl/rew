use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Char {
    Raw(char),
    Escaped(char, char),
}

impl Char {
    pub fn value(&self) -> char {
        match self {
            Char::Raw(ch) => *ch,
            Char::Escaped(_, ch) => *ch,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Char::Raw(_) => 1,
            Char::Escaped(_, _) => 2,
        }
    }

    pub fn raw_vec(string: &str) -> Vec<Char> {
        string.chars().map(Char::Raw).collect()
    }

    pub fn join(chars: &[Char]) -> String {
        chars.iter().map(Char::value).collect()
    }

    pub fn sum_len(chars: &[Char]) -> usize {
        chars.iter().fold(0, |sum, ch| sum + ch.len())
    }
}

impl fmt::Display for Char {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Char::Raw(ch) => write!(formatter, "'{}'", ch),
            Char::Escaped(esc, ch) => write!(formatter, "escape sequence '{}{}'", esc, ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_value() {
        assert_eq!(Char::Raw('a').value(), 'a');
    }

    #[test]
    fn raw_len() {
        assert_eq!(Char::Raw('a').len(), 1);
    }

    #[test]
    fn raw_fmt() {
        assert_eq!(format!("{}", Char::Raw('a')), "'a'");
    }

    #[test]
    fn escaped_value() {
        assert_eq!(Char::Escaped('|', '}').value(), '}');
    }

    #[test]
    fn escaped_len() {
        assert_eq!(Char::Escaped('|', '}').len(), 2);
    }

    #[test]
    fn escaped_fmt() {
        assert_eq!(
            format!("{}", Char::Escaped('|', '}')),
            "escape sequence '|}'"
        );
    }

    #[test]
    fn raw_vec() {
        assert_eq!(Char::raw_vec("ab"), vec![Char::Raw('a'), Char::Raw('b')]);
    }

    #[test]
    fn join() {
        let chars = [Char::Raw('a'), Char::Escaped('|', '}')];
        assert_eq!(Char::join(&chars), "a}".to_string());
    }

    #[test]
    fn sum_len() {
        let chars = [Char::Raw('a'), Char::Escaped('|', '}')];
        assert_eq!(Char::sum_len(&chars), 3);
    }
}
