use crate::pattern::types::{Range, Transform};
use std::str::CharIndices;

impl Transform {
    pub fn apply(&self, value: &mut String) {
        match self {
            Transform::Substring(Range { offset, length }) => {
                if *offset > 0 {
                    if let Some((start, _)) = value.char_indices().nth(*offset) {
                        value.replace_range(..start, "");
                    } else {
                        value.clear();
                    }
                }
                if *length > 0 {
                    if let Some((end, _)) = value.char_indices().nth(*length) {
                        value.replace_range(end.., "");
                    }
                }
            }
            Transform::SubstringFromEnd(Range { offset, length }) => {
                if *offset > 0 {
                    if let Some((start, _)) = value.char_indices().rev().nth(*offset - 1) {
                        value.replace_range(start.., "");
                    } else {
                        value.clear();
                    }
                }
                if *length > 0 {
                    if let Some((end, _)) = value.char_indices().rev().nth(*length - 1) {
                        value.replace_range(..end, "");
                    }
                }
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
        let mut value = "ábčd".to_string();
        Transform::Substring(Range {
            offset: 0,
            length: 0,
        })
        .apply(&mut value);
        assert_eq!(value, "ábčd");
    }

    #[test]
    fn apply_substring_offset() {
        let mut value = "ábčd".to_string();
        Transform::Substring(Range {
            offset: 3,
            length: 0,
        })
        .apply(&mut value);
        assert_eq!(value, "d");
    }

    #[test]
    fn apply_substring_offset_over() {
        let mut value = "ábčd".to_string();
        Transform::Substring(Range {
            offset: 4,
            length: 0,
        })
        .apply(&mut value);
        assert_eq!(value, "");
    }

    #[test]
    fn apply_substring_length() {
        let mut value = "ábčd".to_string();
        Transform::Substring(Range {
            offset: 0,
            length: 3,
        })
        .apply(&mut value);
        assert_eq!(value, "ábč");
    }

    #[test]
    fn apply_substring_length_max() {
        let mut value = "ábčd".to_string();
        Transform::Substring(Range {
            offset: 0,
            length: 4,
        })
        .apply(&mut value);
        assert_eq!(value, "ábčd");
    }

    #[test]
    fn apply_substring_length_over() {
        let mut value = "ábčd".to_string();
        Transform::Substring(Range {
            offset: 0,
            length: 5,
        })
        .apply(&mut value);
        assert_eq!(value, "ábčd");
    }

    #[test]
    fn apply_substring_offset_length() {
        let mut value = "ábčd".to_string();
        Transform::Substring(Range {
            offset: 1,
            length: 1,
        })
        .apply(&mut value);
        assert_eq!(value, "b");
    }

    #[test]
    fn apply_substring_offset_over_length() {
        let mut value = "ábčd".to_string();
        Transform::Substring(Range {
            offset: 4,
            length: 1,
        })
        .apply(&mut value);
        assert_eq!(value, "");
    }

    #[test]
    fn apply_substring_offset_length_over() {
        let mut value = "ábčd".to_string();
        Transform::Substring(Range {
            offset: 1,
            length: 4,
        })
        .apply(&mut value);
        assert_eq!(value, "bčd");
    }

    #[test]
    fn apply_substring_fe_full() {
        let mut value = "ábčd".to_string();
        Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 0,
        })
        .apply(&mut value);
        assert_eq!(value, "ábčd");
    }

    #[test]
    fn apply_substring_fe_offset() {
        let mut value = "ábčd".to_string();
        Transform::SubstringFromEnd(Range {
            offset: 3,
            length: 0,
        })
        .apply(&mut value);
        assert_eq!(value, "á");
    }

    #[test]
    fn apply_substring_fe_offset_over() {
        let mut value = "ábčd".to_string();
        Transform::SubstringFromEnd(Range {
            offset: 4,
            length: 0,
        })
        .apply(&mut value);
        assert_eq!(value, "");
    }

    #[test]
    fn apply_substring_fe_length() {
        let mut value = "ábčd".to_string();
        Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 3,
        })
        .apply(&mut value);
        assert_eq!(value, "bčd");
    }

    #[test]
    fn apply_substring_fe_length_max() {
        let mut value = "ábčd".to_string();
        Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 4,
        })
        .apply(&mut value);
        assert_eq!(value, "ábčd");
    }

    #[test]
    fn apply_substring_fe_length_over() {
        let mut value = "ábčd".to_string();
        Transform::SubstringFromEnd(Range {
            offset: 0,
            length: 5,
        })
        .apply(&mut value);
        assert_eq!(value, "ábčd");
    }

    #[test]
    fn apply_substring_fe_offset_length() {
        let mut value = "ábčd".to_string();
        Transform::SubstringFromEnd(Range {
            offset: 1,
            length: 1,
        })
        .apply(&mut value);
        assert_eq!(value, "č");
    }

    #[test]
    fn apply_substring_fe_offset_over_length() {
        let mut value = "ábčd".to_string();
        Transform::SubstringFromEnd(Range {
            offset: 4,
            length: 1,
        })
        .apply(&mut value);
        assert_eq!(value, "");
    }

    #[test]
    fn apply_substring_fe_offset_length_over() {
        let mut value = "ábčd".to_string();
        Transform::SubstringFromEnd(Range {
            offset: 1,
            length: 4,
        })
        .apply(&mut value);
        assert_eq!(value, "ábč");
    }
}
