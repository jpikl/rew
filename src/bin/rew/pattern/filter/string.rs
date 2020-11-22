use crate::pattern::filter::error::Result;

pub fn replace_first(value: String, target: &str, replacement: &str) -> Result {
    Ok(value.replacen(target, replacement, 1))
}

pub fn replace_all(value: String, target: &str, replacement: &str) -> Result {
    Ok(value.replace(target, replacement))
}

pub fn replace_empty(mut value: String, replacement: &str) -> Result {
    if value.is_empty() {
        value.push_str(replacement);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod replace_first {
        use super::*;

        #[test]
        fn empty_with_empty() {
            assert_eq!(replace_first(String::new(), "ab", ""), Ok(String::new()));
        }

        #[test]
        fn empty_with_nonempty() {
            assert_eq!(replace_first(String::new(), "ab", "x"), Ok(String::new()));
        }

        #[test]
        fn none_with_empty() {
            assert_eq!(
                replace_first(String::from("cd"), "ab", ""),
                Ok(String::from("cd"))
            );
        }

        #[test]
        fn none_with_nonempty() {
            assert_eq!(
                replace_first(String::from("cd"), "ab", "x"),
                Ok(String::from("cd"))
            );
        }

        #[test]
        fn first_with_empty() {
            assert_eq!(
                replace_first(String::from("abcd_abcd"), "ab", ""),
                Ok(String::from("cd_abcd"))
            );
        }

        #[test]
        fn first_with_nonempty() {
            assert_eq!(
                replace_first(String::from("abcd_abcd"), "ab", "x"),
                Ok(String::from("xcd_abcd"))
            );
        }
    }

    mod replace_all {
        use super::*;

        #[test]
        fn empty_with_empty() {
            assert_eq!(replace_all(String::new(), "ab", ""), Ok(String::new()));
        }

        #[test]
        fn empty_with_nonempty() {
            assert_eq!(replace_all(String::new(), "ab", "x"), Ok(String::new()));
        }

        #[test]
        fn none_with_empty() {
            assert_eq!(
                replace_all(String::from("cd"), "ab", ""),
                Ok(String::from("cd"))
            );
        }

        #[test]
        fn none_with_nonempty() {
            assert_eq!(
                replace_all(String::from("cd"), "ab", "x"),
                Ok(String::from("cd"))
            );
        }

        #[test]
        fn first_with_empty() {
            assert_eq!(
                replace_all(String::from("abcd_abcd"), "ab", ""),
                Ok(String::from("cd_cd"))
            );
        }

        #[test]
        fn first_with_nonempty() {
            assert_eq!(
                replace_all(String::from("abcd_abcd"), "ab", "x"),
                Ok(String::from("xcd_xcd"))
            );
        }
    }

    mod replace_empty {
        use super::*;

        #[test]
        fn empty_with_empty() {
            assert_eq!(replace_empty(String::new(), ""), Ok(String::new()));
        }

        #[test]
        fn empty_with_nonempty() {
            assert_eq!(replace_empty(String::new(), "def"), Ok(String::from("def")));
        }

        #[test]
        fn nonempty_with_empty() {
            assert_eq!(
                replace_empty(String::from("abc"), ""),
                Ok(String::from("abc"))
            );
        }

        #[test]
        fn nonempty_with_nonempty() {
            assert_eq!(
                replace_empty(String::from("abc"), "def"),
                Ok(String::from("abc"))
            );
        }
    }
}
