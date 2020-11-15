use crate::pattern::filter::error::Result;
use unidecode::unidecode;

pub fn trim(value: String) -> Result {
    Ok(value.trim().to_string())
}

pub fn to_lowercase(value: String) -> Result {
    Ok(value.to_lowercase())
}

pub fn to_uppercase(value: String) -> Result {
    Ok(value.to_uppercase())
}

pub fn to_ascii(value: String) -> Result {
    Ok(unidecode(&value))
}

pub fn remove_non_ascii(mut value: String) -> Result {
    value.retain(|ch| ch.is_ascii());
    Ok(value)
}

pub fn left_pad(mut value: String, padding: &str) -> Result {
    for char in padding.chars().rev().skip(value.len()) {
        value.insert(0, char);
    }
    Ok(value)
}

pub fn left_pad_repeat(value: String, padding: &str, count: usize) -> Result {
    left_pad(value, &padding.repeat(count))
}

pub fn right_pad(mut value: String, padding: &str) -> Result {
    for char in padding.chars().skip(value.len()) {
        value.push(char);
    }
    Ok(value)
}

pub fn right_pad_repeat(value: String, padding: &str, count: usize) -> Result {
    right_pad(value, &padding.repeat(count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_not_trimmed() {
        assert_eq!(trim(String::from("  abc  ")), Ok(String::from("abc")));
    }

    #[test]
    fn trim_trimmed() {
        assert_eq!(trim(String::from("abc")), Ok(String::from("abc")));
    }

    #[test]
    fn trim_empty() {
        assert_eq!(trim(String::new()), Ok(String::new()));
    }

    #[test]
    fn convert_to_lowercase() {
        assert_eq!(
            to_lowercase(String::from("ábčdÁBČD")),
            Ok(String::from("ábčdábčd"))
        );
    }

    #[test]
    fn convert_empty_to_lowercase() {
        assert_eq!(to_lowercase(String::new()), Ok(String::new()));
    }

    #[test]
    fn convert_some_to_uppercase() {
        assert_eq!(
            to_uppercase(String::from("ábčdÁBČD")),
            Ok(String::from("ÁBČDÁBČD"))
        );
    }

    #[test]
    fn convert_empty_to_uppercase() {
        assert_eq!(to_uppercase(String::new()), Ok(String::new()));
    }

    #[test]
    fn convert_some_to_ascii() {
        assert_eq!(
            to_ascii(String::from("ábčdÁBČD")),
            Ok(String::from("abcdABCD"))
        );
    }

    #[test]
    fn convert_empty_to_ascii() {
        assert_eq!(to_ascii(String::new()), Ok(String::new()));
    }

    #[test]
    fn remove_non_ascii_from_some() {
        assert_eq!(
            remove_non_ascii(String::from("ábčdÁBČD")),
            Ok(String::from("bdBD"))
        );
    }

    #[test]
    fn remove_non_ascii_from_empty() {
        assert_eq!(remove_non_ascii(String::new()), Ok(String::new()));
    }

    #[test]
    fn left_pad_empty() {
        assert_eq!(left_pad(String::new(), "0123"), Ok(String::from("0123")));
    }

    #[test]
    fn left_pad_some() {
        assert_eq!(
            left_pad(String::from("ab"), "0123"),
            Ok(String::from("01ab"))
        );
    }

    #[test]
    fn left_pad_none() {
        assert_eq!(
            left_pad(String::from("abcd"), "0123"),
            Ok(String::from("abcd"))
        );
    }

    #[test]
    fn left_pad_with_empty() {
        assert_eq!(left_pad(String::from("abcd"), ""), Ok(String::from("abcd")));
    }

    #[test]
    fn left_pad_repeated() {
        assert_eq!(
            left_pad_repeat(String::from("a"), "01", 2),
            Ok(String::from("010a"))
        );
    }

    #[test]
    fn left_pad_repeated_zero_times() {
        assert_eq!(
            left_pad_repeat(String::from("a"), "01", 0),
            Ok(String::from("a"))
        );
    }

    #[test]
    fn right_pad_empty() {
        assert_eq!(right_pad(String::new(), "0123"), Ok(String::from("0123")));
    }

    #[test]
    fn right_pad_some() {
        assert_eq!(
            right_pad(String::from("ab"), "0123"),
            Ok(String::from("ab23"))
        );
    }

    #[test]
    fn right_pad_none() {
        assert_eq!(
            right_pad(String::from("abcd"), "0123"),
            Ok(String::from("abcd"))
        );
    }

    #[test]
    fn right_pad_with_empty() {
        assert_eq!(
            right_pad(String::from("abcd"), ""),
            Ok(String::from("abcd"))
        );
    }

    #[test]
    fn right_pad_repeated() {
        assert_eq!(
            right_pad_repeat(String::from("a"), "01", 2),
            Ok(String::from("a101"))
        );
    }

    #[test]
    fn right_pad_repeated_zero_times() {
        assert_eq!(
            right_pad_repeat(String::from("a"), "01", 0),
            Ok(String::from("a"))
        );
    }
}
