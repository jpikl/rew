use crate::pattern::types::{Range, Substitution, Transform};
use std::str::CharIndices;

impl Transform {
    pub fn apply(&self, mut string: String) -> String {
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
            _ => {
                panic!("Not implemented");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn apply_substring_full() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 0,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_offset() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 3,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "d");
    }

    #[test]
    fn apply_substring_offset_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 4,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_length() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 0,
            length: 3,
        })
        .apply(string);
        assert_eq!(string, "ábč");
    }

    #[test]
    fn apply_substring_length_max() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 0,
            length: 4,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_length_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 0,
            length: 5,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_offset_length() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 1,
            length: 1,
        })
        .apply(string);
        assert_eq!(string, "b");
    }

    #[test]
    fn apply_substring_offset_over_length() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 4,
            length: 1,
        })
        .apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_offset_length_over() {
        let mut string = "ábčd".to_string();
        string = Transform::Substring(Range {
            offset: 1,
            length: 4,
        })
        .apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn apply_substring_fe_full() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_fe_offset() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 3,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "á");
    }

    #[test]
    fn apply_substring_fe_offset_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 4,
            length: 0,
        })
        .apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_fe_length() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 3,
        })
        .apply(string);
        assert_eq!(string, "bčd");
    }

    #[test]
    fn apply_substring_fe_length_max() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 4,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_fe_length_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 5,
        })
        .apply(string);
        assert_eq!(string, "ábčd");
    }

    #[test]
    fn apply_substring_fe_offset_length() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 1,
            length: 1,
        })
        .apply(string);
        assert_eq!(string, "č");
    }

    #[test]
    fn apply_substring_fe_offset_over_length() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 4,
            length: 1,
        })
        .apply(string);
        assert_eq!(string, "");
    }

    #[test]
    fn apply_substring_fe_offset_length_over() {
        let mut string = "ábčd".to_string();
        string = Transform::SubstringFromEnd(Range {
            offset: 1,
            length: 4,
        })
        .apply(string);
        assert_eq!(string, "ábč");
    }

    #[test]
    fn apply_replace_first() {
        let mut string = "abcd_abcd".to_string();
        string = Transform::ReplaceFirst(Substitution {
            value: "ab".to_string(),
            replacement: "x".to_string(),
        })
        .apply(string);
        assert_eq!(string, "xcd_abcd");
    }

    #[test]
    fn apply_replace_all() {
        let mut string = "abcd_abcd".to_string();
        string = Transform::ReplaceAll(Substitution {
            value: "ab".to_string(),
            replacement: "x".to_string(),
        })
        .apply(string);
        assert_eq!(string, "xcd_xcd");
    }

    #[test]
    fn apply_remove_first() {
        let mut string = "abcd_abcd".to_string();
        string = Transform::ReplaceFirst(Substitution {
            value: "ab".to_string(),
            replacement: String::new(),
        })
        .apply(string);
        assert_eq!(string, "cd_abcd");
    }

    #[test]
    fn apply_remove_all() {
        let mut string = "abcd_abcd".to_string();
        string = Transform::ReplaceAll(Substitution {
            value: "ab".to_string(),
            replacement: String::new(),
        })
        .apply(string);
        assert_eq!(string, "cd_cd");
    }
}
