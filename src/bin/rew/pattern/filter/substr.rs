use crate::pattern::filter::error::Result;
use crate::pattern::range::Index;

pub fn get_forward(mut value: String, start: Index, length: Option<Index>) -> Result {
    if let Some((start, _)) = value.char_indices().nth(start) {
        value.replace_range(..start, "");
    } else {
        value.clear();
    }

    if let Some(length) = length {
        if let Some((end, _)) = value.char_indices().nth(length) {
            value.replace_range(end.., "");
        }
    }

    Ok(value)
}

pub fn get_backward(mut value: String, start: Index, length: Option<Index>) -> Result {
    if start > 0 {
        if let Some((start, _)) = value.char_indices().nth_back(start - 1) {
            value.replace_range(start.., "");
        } else {
            value.clear();
        }
    }

    if let Some(length) = length {
        if length > 0 {
            if let Some((end, _)) = value.char_indices().nth_back(length - 1) {
                value.replace_range(..end, "");
            }
        } else {
            value.clear();
        }
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_forward {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(get_forward(String::new(), 0, None), Ok(String::new()));
        }

        #[test]
        fn from_first() {
            assert_eq!(
                get_forward(String::from("ábčd"), 0, None),
                Ok(String::from("ábčd"))
            );
        }

        #[test]
        fn from_last() {
            assert_eq!(
                get_forward(String::from("ábčd"), 3, None),
                Ok(String::from("d"))
            );
        }

        #[test]
        fn from_over() {
            assert_eq!(
                get_forward(String::from("ábčd"), 4, None),
                Ok(String::from(""))
            );
        }

        #[test]
        fn to_below() {
            assert_eq!(
                get_forward(String::from("ábčd"), 0, Some(0)),
                Ok(String::from(""))
            );
        }

        #[test]
        fn to_last_but_one() {
            assert_eq!(
                get_forward(String::from("ábčd"), 0, Some(3)),
                Ok(String::from("ábč"))
            );
        }

        #[test]
        fn to_last() {
            assert_eq!(
                get_forward(String::from("ábčd"), 0, Some(4)),
                Ok(String::from("ábčd"))
            );
        }

        #[test]
        fn to_over() {
            assert_eq!(
                get_forward(String::from("ábčd"), 0, Some(5)),
                Ok(String::from("ábčd"))
            );
        }

        #[test]
        fn from_first_to_below() {
            assert_eq!(
                get_forward(String::from("ábčd"), 0, Some(0)),
                Ok(String::from(""))
            );
        }

        #[test]
        fn from_first_to_last_but_one() {
            assert_eq!(
                get_forward(String::from("ábčd"), 0, Some(3)),
                Ok(String::from("ábč"))
            );
        }

        #[test]
        fn from_first_to_last() {
            assert_eq!(
                get_forward(String::from("ábčd"), 0, Some(4)),
                Ok(String::from("ábčd"))
            );
        }

        #[test]
        fn from_last_to_last() {
            assert_eq!(
                get_forward(String::from("ábčd"), 3, Some(1)),
                Ok(String::from("d"))
            );
        }

        #[test]
        fn from_last_to_over() {
            assert_eq!(
                get_forward(String::from("ábčd"), 3, Some(2)),
                Ok(String::from("d"))
            );
        }

        #[test]
        fn from_over_to_over() {
            assert_eq!(
                get_forward(String::from("ábčd"), 4, Some(1)),
                Ok(String::from(""))
            );
        }
    }

    mod get_backward {
        use super::*;

        #[test]
        fn empty() {
            assert_eq!(get_backward(String::new(), 0, None), Ok(String::new()));
        }

        #[test]
        fn from_first() {
            assert_eq!(
                get_backward(String::from("ábčd"), 0, None),
                Ok(String::from("ábčd"))
            );
        }

        #[test]
        fn from_last() {
            assert_eq!(
                get_backward(String::from("ábčd"), 3, None),
                Ok(String::from("á"))
            );
        }

        #[test]
        fn from_over() {
            assert_eq!(
                get_backward(String::from("ábčd"), 4, None),
                Ok(String::from(""))
            );
        }

        #[test]
        fn to_below() {
            assert_eq!(
                get_backward(String::from("ábčd"), 0, Some(0)),
                Ok(String::from(""))
            );
        }

        #[test]
        fn to_last_but_one() {
            assert_eq!(
                get_backward(String::from("ábčd"), 0, Some(3)),
                Ok(String::from("bčd"))
            );
        }

        #[test]
        fn to_last() {
            assert_eq!(
                get_backward(String::from("ábčd"), 0, Some(4)),
                Ok(String::from("ábčd"))
            );
        }

        #[test]
        fn to_over() {
            assert_eq!(
                get_backward(String::from("ábčd"), 0, Some(5)),
                Ok(String::from("ábčd"))
            );
        }

        #[test]
        fn from_first_to_below() {
            assert_eq!(
                get_backward(String::from("ábčd"), 0, Some(0)),
                Ok(String::from(""))
            );
        }

        #[test]
        fn from_first_to_last_but_one() {
            assert_eq!(
                get_backward(String::from("ábčd"), 0, Some(3)),
                Ok(String::from("bčd"))
            );
        }

        #[test]
        fn from_first_to_last() {
            assert_eq!(
                get_backward(String::from("ábčd"), 0, Some(4)),
                Ok(String::from("ábčd"))
            );
        }

        #[test]
        fn from_last_to_last() {
            assert_eq!(
                get_backward(String::from("ábčd"), 3, Some(1)),
                Ok(String::from("á"))
            );
        }

        #[test]
        fn from_last_to_over() {
            assert_eq!(
                get_backward(String::from("ábčd"), 3, Some(2)),
                Ok(String::from("á"))
            );
        }

        #[test]
        fn from_over_to_over() {
            assert_eq!(
                get_backward(String::from("ábčd"), 4, Some(1)),
                Ok(String::from(""))
            );
        }

        #[test]
        fn from_extra_over_to_over() {
            // Covers different evaluation branch than from_over_to_over
            assert_eq!(
                get_backward(String::from("ábčd"), 5, Some(1)),
                Ok(String::from(""))
            );
        }
    }
}
