use crate::pattern::filter::error::Result;
use crate::pattern::range::Range;

pub fn get_forward(mut value: String, range: &Range) -> Result {
    if let Some(start) = range.start() {
        if let Some((start, _)) = value.char_indices().nth(start) {
            value.replace_range(..start, "");
        } else {
            value.clear();
        }
    }

    if let Some(length) = range.length() {
        if let Some((end, _)) = value.char_indices().nth(length) {
            value.replace_range(end.., "");
        }
    }

    Ok(value)
}

pub fn get_backward(mut value: String, range: &Range) -> Result {
    if let Some(start) = range.start() {
        if start > 0 {
            if let Some((start, _)) = value.char_indices().nth_back(start - 1) {
                value.replace_range(start.., "");
            } else {
                value.clear();
            }
        }
    }
    if let Some(length) = range.length() {
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

    #[test]
    fn forward_from_empty() {
        assert_eq!(
            get_forward(String::new(), &Range::From(0)),
            Ok(String::new())
        );
    }

    #[test]
    fn forward_from_first() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::From(0)),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn forward_from_last() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::From(3)),
            Ok(String::from("d"))
        );
    }

    #[test]
    fn forward_from_over() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::From(4)),
            Ok(String::from(""))
        );
    }

    #[test]
    fn forward_to_below() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::To(0)),
            Ok(String::from(""))
        );
    }

    #[test]
    fn forward_to_last_but_one() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::To(3)),
            Ok(String::from("ábč"))
        );
    }

    #[test]
    fn forward_to_last() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::To(4)),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn forward_to_over() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::To(5)),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn forward_from_first_to_below() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::FromTo(0, 0)),
            Ok(String::from(""))
        );
    }

    #[test]
    fn forward_from_first_to_last_but_one() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::FromTo(0, 3)),
            Ok(String::from("ábč"))
        );
    }

    #[test]
    fn forward_from_first_to_last() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::FromTo(0, 4)),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn forward_from_last_to_last() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::FromTo(3, 4)),
            Ok(String::from("d"))
        );
    }

    #[test]
    fn forward_from_last_to_over() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::FromTo(3, 5)),
            Ok(String::from("d"))
        );
    }

    #[test]
    fn forward_from_over_to_over() {
        assert_eq!(
            get_forward(String::from("ábčd"), &Range::FromTo(4, 5)),
            Ok(String::from(""))
        );
    }

    #[test]
    fn backward_from_first() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::From(0)),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn backward_from_last() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::From(3)),
            Ok(String::from("á"))
        );
    }

    #[test]
    fn backward_from_empty() {
        assert_eq!(
            get_backward(String::new(), &Range::From(0)),
            Ok(String::new())
        );
    }

    #[test]
    fn backward_from_over() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::From(4)),
            Ok(String::from(""))
        );
    }

    #[test]
    fn backward_to_below() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::To(0)),
            Ok(String::from(""))
        );
    }

    #[test]
    fn backward_to_last_but_one() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::To(3)),
            Ok(String::from("bčd"))
        );
    }

    #[test]
    fn backward_to_last() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::To(4)),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn backward_to_over() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::To(5)),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn backward_from_first_to_below() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::FromTo(0, 0)),
            Ok(String::from(""))
        );
    }

    #[test]
    fn backward_from_first_to_last_but_one() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::FromTo(0, 3)),
            Ok(String::from("bčd"))
        );
    }

    #[test]
    fn backward_from_first_to_last() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::FromTo(0, 4)),
            Ok(String::from("ábčd"))
        );
    }

    #[test]
    fn backward_from_last_to_last() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::FromTo(3, 4)),
            Ok(String::from("á"))
        );
    }

    #[test]
    fn backward_from_last_to_over() {
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::FromTo(3, 5)),
            Ok(String::from("á"))
        );
    }

    #[test]
    fn backward_from_over_to_over() {
        // Each assert covers different evaluation branch
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::FromTo(4, 5)),
            Ok(String::from(""))
        );
        assert_eq!(
            get_backward(String::from("ábčd"), &Range::FromTo(5, 6)),
            Ok(String::from(""))
        );
    }
}
