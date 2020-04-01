use crate::pattern::range::Range;
use crate::pattern::substitution::Substitution;
use crate::pattern::transform::Transform;
use unidecode::unidecode;

impl Transform {
    fn apply(&self, mut string: String) -> String {
        match self {
            Transform::Substring(Range { offset, length }) => {
                if *offset > 0 {
                    if let Some((start, _)) = string.char_indices().nth(*offset) {
                        string.replace_range(..start, "");
                    } else {
                        string.clear();
                    }
                }
                if *length > 0 {
                    if let Some((end, _)) = string.char_indices().nth(*length) {
                        string.replace_range(end.., "");
                    }
                }
                string
            }
            Transform::SubstringFromEnd(Range { offset, length }) => {
                if *offset > 0 {
                    if let Some((start, _)) = string.char_indices().rev().nth(*offset - 1) {
                        string.replace_range(start.., "");
                    } else {
                        string.clear();
                    }
                }
                if *length > 0 {
                    if let Some((end, _)) = string.char_indices().rev().nth(*length - 1) {
                        string.replace_range(..end, "");
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
            Transform::LeftPad(pad_chars) => {
                for pad_char in pad_chars.iter().rev().skip(string.len()) {
                    string.insert(0, *pad_char);
                }
                string
            }
            Transform::RightPad(pad_chars) => {
                for pad_char in pad_chars.iter().skip(string.len()) {
                    string.push(*pad_char);
                }
                string
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::substitution::Substitution;

    #[test]
    fn substring_full() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 0,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_offset() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 3,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn substring_offset_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 4,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_length() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 0,
            length: 3,
        })
        .apply(string);
        assert_eq!(string, "ábč");
    }

    #[test]
    fn substring_length_max() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 0,
            length: 4,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_length_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 0,
            length: 5,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_offset_length() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 1,
            length: 1,
        })
        .apply(string);
        assert_eq!(string, "b");
    }

    #[test]
    fn substring_offset_over_length() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 4,
            length: 1,
        })
        .apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_offset_length_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 1,
            length: 4,
        })
        .apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn substring_fe_full() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_fe_offset() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 3,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn substring_fe_offset_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 4,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_fe_length() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 3,
        })
        .apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn substring_fe_length_max() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 4,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_fe_length_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 5,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn substring_fe_offset_length() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 1,
            length: 1,
        })
        .apply(string);
        assert_eq!(string, "č");
    }

    #[test]
    fn substring_fe_offset_over_length() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 4,
            length: 1,
        })
        .apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn substring_fe_offset_length_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 1,
            length: 4,
        })
        .apply(string);
        assert_eq!(string, "ábč");
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
        string = Transform::LeftPad(vec!['0', '1', '2', '3']).apply(string);
        assert_eq!(string, "0123");
    }

    #[test]
    fn left_pad_some() {
        let mut string = "ab".to_string();
        string = Transform::LeftPad(vec!['0', '1', '2', '3']).apply(string);
        assert_eq!(string, "01ab");
    }

    #[test]
    fn left_pad_none() {
        let mut string = "abcd".to_string();
        string = Transform::LeftPad(vec!['0', '1', '2', '3']).apply(string);
        assert_eq!(string, "abcd");
    }

    #[test]
    fn right_pad_all() {
        let mut string = "".to_string();
        string = Transform::RightPad(vec!['0', '1', '2', '3']).apply(string);
        assert_eq!(string, "0123");
    }

    #[test]
    fn right_pad_some() {
        let mut string = "ab".to_string();
        string = Transform::RightPad(vec!['0', '1', '2', '3']).apply(string);
        assert_eq!(string, "ab23");
    }

    #[test]
    fn right_pad_none() {
        let mut string = "abcd".to_string();
        string = Transform::RightPad(vec!['0', '1', '2', '3']).apply(string);
        assert_eq!(string, "abcd");
    }
}
