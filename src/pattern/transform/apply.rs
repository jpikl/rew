use crate::pattern::substitution::Substitution;
use crate::pattern::transform::Transform;
use unidecode::unidecode;

impl Transform {
    pub fn apply(&self, mut string: String) -> String {
        match self {
            Transform::Substring(range) => {
                if let Some(start) = range.start() {
                    if let Some((start, _)) = string.char_indices().nth(start) {
                        string.replace_range(..start, "");
                    } else {
                        string.clear();
                    }
                }
                if let Some(length) = range.length() {
                    if let Some((end, _)) = string.char_indices().nth(length) {
                        string.replace_range(end.., "");
                    }
                }
                string
            }

            Transform::SubstringReverse(range) => {
                if let Some(start) = range.start() {
                    if start > 0 {
                        if let Some((start, _)) = string.char_indices().nth_back(start - 1) {
                            string.replace_range(start.., "");
                        } else {
                            string.clear();
                        }
                    }
                }
                if let Some(length) = range.length() {
                    if length > 0 {
                        if let Some((end, _)) = string.char_indices().nth_back(length - 1) {
                            string.replace_range(..end, "");
                        }
                    } else {
                        string.clear();
                    }
                }
                string
            }

            Transform::ReplaceFirst(Substitution { value, replacement }) => {
                string.replacen(value, replacement, 1)
            }

            Transform::ReplaceAll(Substitution { value, replacement }) => {
                string.replace(value, replacement)
            }

            Transform::Trim => string.trim().to_string(),
            Transform::Lowercase => string.to_lowercase(),
            Transform::Uppercase => string.to_uppercase(),
            Transform::ToAscii => unidecode(&string),

            Transform::RemoveNonAscii => {
                string.retain(|ch| ch.is_ascii());
                string
            }

            Transform::LeftPad(padding) => {
                for char in padding.chars().rev().skip(string.len()) {
                    string.insert(0, char);
                }
                string
            }

            Transform::RightPad(padding) => {
                for char in padding.chars().skip(string.len()) {
                    string.push(char);
                }
                string
            }

            Transform::Default(default) => {
                if string.is_empty() {
                    string.push_str(default);
                }
                string
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::range::Range;
    use crate::pattern::substitution::Substitution;

    #[test]
    fn substring_full() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::Full).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_from_first() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::From(0)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_from_last() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::From(3)).apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn substring_from_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::From(4)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_to_below() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::To(0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_to_last_but_one() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::To(3)).apply(string);
        assert_eq!(string, "ábč");
    }

    #[test]
    fn substring_to_last() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::To(4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_to_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::To(5)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_from_first_to_below() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::FromTo(0, 0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_from_first_to_last_but_one() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::FromTo(0, 3)).apply(string);
        assert_eq!(string, "ábč");
    }

    #[test]
    fn substring_from_first_to_last() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::FromTo(0, 4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_from_last_to_last() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::FromTo(3, 4)).apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn substring_from_last_to_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::FromTo(3, 5)).apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn substring_from_over_to_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range::FromTo(4, 5)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_reverse_full() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::Full).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_reverse_from_first() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::From(0)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_reverse_from_last() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::From(3)).apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn substring_reverse_from_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::From(4)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_reverse_to_below() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::To(0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_reverse_to_last_but_one() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::To(3)).apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn substring_reverse_to_last() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::To(4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_reverse_to_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::To(5)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_reverse_from_first_to_below() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::FromTo(0, 0)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_reverse_from_first_to_last_but_one() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::FromTo(0, 3)).apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn substring_reverse_from_first_to_last() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::FromTo(0, 4)).apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_reverse_from_last_to_last() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::FromTo(3, 4)).apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn substring_reverse_from_last_to_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::FromTo(3, 5)).apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn substring_reverse_from_over_to_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringReverse(Range::FromTo(4, 5)).apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn replace_first() {
        let mut string = "abcd_abcd".to_string();
        string = Transform::ReplaceFirst(Substitution {
            value: "ab".to_string(),
            replacement: "x".to_string(),
        })
        .apply(string);
        assert_eq!(string, "xcd_abcd");
    }

    #[test]
    fn replace_all() {
        let mut string = "abcd_abcd".to_string();
        string = Transform::ReplaceAll(Substitution {
            value: "ab".to_string(),
            replacement: "x".to_string(),
        })
        .apply(string);
        assert_eq!(string, "xcd_xcd");
    }

    #[test]
    fn remove_first() {
        let mut string = "abcd_abcd".to_string();
        string = Transform::ReplaceFirst(Substitution {
            value: "ab".to_string(),
            replacement: String::new(),
        })
        .apply(string);
        assert_eq!(string, "cd_abcd");
    }

    #[test]
    fn remove_all() {
        let mut string = "abcd_abcd".to_string();
        string = Transform::ReplaceAll(Substitution {
            value: "ab".to_string(),
            replacement: String::new(),
        })
        .apply(string);
        assert_eq!(string, "cd_cd");
    }

    #[test]
    fn trim_none() {
        let mut string = "abcd".to_string();
        string = Transform::Trim.apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn trim() {
        let mut string = " abcd ".to_string();
        string = Transform::Trim.apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn lowercase() {
        let mut string = "ábčdÁBČD".to_string();
        string = Transform::Lowercase.apply(string);
        assert_eq!(string, "ábčdábčd");
    }

    #[test]
    fn uppercase() {
        let mut string = "ábčdÁBČD".to_string();
        string = Transform::Uppercase.apply(string);
        assert_eq!(string, "ÁBČDÁBČD");
    }

    #[test]
    fn to_ascii() {
        let mut string = "ábčdÁBČD".to_string();
        string = Transform::ToAscii.apply(string);
        assert_eq!(string, "abcdABCD");
    }

    #[test]
    fn remove_non_ascii() {
        let mut string = "ábčdÁBČD".to_string();
        string = Transform::RemoveNonAscii.apply(string);
        assert_eq!(string, "bdBD");
    }

    #[test]
    fn left_pad_all() {
        let mut string = "".to_string();
        string = Transform::LeftPad("0123".to_string()).apply(string);
        assert_eq!(string, "0123");
    }

    #[test]
    fn left_pad_some() {
        let mut string = "ab".to_string();
        string = Transform::LeftPad("0123".to_string()).apply(string);
        assert_eq!(string, "01ab");
    }

    #[test]
    fn left_pad_none() {
        let mut string = "abcd".to_string();
        string = Transform::LeftPad("0123".to_string()).apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn right_pad_all() {
        let mut string = "".to_string();
        string = Transform::RightPad("0123".to_string()).apply(string);
        assert_eq!(string, "0123");
    }

    #[test]
    fn right_pad_some() {
        let mut string = "ab".to_string();
        string = Transform::RightPad("0123".to_string()).apply(string);
        assert_eq!(string, "ab23");
    }

    #[test]
    fn right_pad_none() {
        let mut string = "abcd".to_string();
        string = Transform::RightPad("0123".to_string()).apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn default_used() {
        let mut string = "".to_string();
        string = Transform::Default("xyz".to_string()).apply(string);
        assert_eq!(string, "xyz");
    }

    #[test]
    fn default_unused() {
        let mut string = "abcd".to_string();
        string = Transform::Default("xyz".to_string()).apply(string);
        assert_eq!(string, "abcd");
    }
}
