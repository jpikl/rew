pub type EscapeSequence = [char; 2];

#[derive(Debug, PartialEq, Clone)]
pub enum Char {
    Raw(char),
    Escaped(char, EscapeSequence),
}

impl Char {
    pub fn value(&self) -> char {
        match self {
            Char::Raw(value) => *value,
            Char::Escaped(value, _) => *value,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Char::Raw(value) => value.len_utf8(),
            Char::Escaped(_, esc_seq) => esc_seq[0].len_utf8() + esc_seq[1].len_utf8(),
        }
    }

    pub fn raw_vec(string: &str) -> Vec<Char> {
        string.chars().map(Char::Raw).collect()
    }

    pub fn join(chars: &[Char]) -> String {
        chars.iter().map(Char::value).collect()
    }

    pub fn sum_len(chars: &[Char]) -> usize {
        chars.iter().fold(0, |sum, char| sum + char.len())
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
        assert_eq!(Char::Raw('á').len(), 2);
    }

    #[test]
    fn escaped_value() {
        assert_eq!(Char::Escaped('a', ['b', 'c']).value(), 'a');
    }

    #[test]
    fn escaped_len() {
        assert_eq!(Char::Escaped('a', ['b', 'c']).len(), 2);
        assert_eq!(Char::Escaped('a', ['á', 'č']).len(), 4);
    }

    #[test]
    fn raw_vec() {
        assert_eq!(Char::raw_vec("ab"), vec![Char::Raw('a'), Char::Raw('b')]);
    }

    #[test]
    fn join() {
        let chars = [Char::Raw('a'), Char::Escaped('b', ['c', 'd'])];
        assert_eq!(Char::join(&chars), "ab".to_string());
    }

    #[test]
    fn sum_len() {
        let chars = [Char::Raw('a'), Char::Escaped('b', ['c', 'd'])];
        assert_eq!(Char::sum_len(&chars), 3);
    }
}
