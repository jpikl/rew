#[derive(Debug, PartialEq)]
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

    pub fn escape(&self) -> Option<char> {
        match self {
            Char::Raw(_) => None,
            Char::Escaped(esc, _) => Some(*esc),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_value() {
        assert_eq!(Char::Raw('a').value(), 'a');
    }

    #[test]
    fn raw_escape() {
        assert_eq!(Char::Raw('a').escape(), None);
    }

    #[test]
    fn raw_len() {
        assert_eq!(Char::Raw('a').len(), 1);
    }

    #[test]
    fn escaped_value() {
        assert_eq!(Char::Escaped('|', '}').value(), '}');
    }

    #[test]
    fn escaped_escape() {
        assert_eq!(Char::Escaped('|', '}').escape(), Some('|'));
    }

    #[test]
    fn escaped_len() {
        assert_eq!(Char::Escaped('|', '}').len(), 2);
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
