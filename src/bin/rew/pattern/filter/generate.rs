use crate::pattern::filter::error::Result;
use crate::pattern::range::Number;
use rand::thread_rng;
use rand::Rng;
use uuid::Uuid;

pub fn repetition(value: &str, count: usize) -> Result {
    Ok(value.repeat(count))
}

pub fn counter(value: u32) -> Result {
    Ok(value.to_string())
}

pub fn random_number(start: Number, end: Option<Number>) -> Result {
    let end = end.unwrap_or(Number::MAX);
    let result = if start == 0 && end == Number::MAX {
        thread_rng().gen() // gen_range(start..=end) would cause an overflow in rand lib
    } else {
        thread_rng().gen_range(start..=end)
    };
    Ok(result.to_string())
}

pub fn random_uuid() -> Result {
    let mut buffer = Uuid::encode_buffer();
    let str = Uuid::new_v4().to_hyphenated().encode_lower(&mut buffer);
    Ok((*str).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::*;

    mod repetition {
        use super::*;

        #[test]
        fn empty_zero_times() {
            assert_eq!(repetition("", 0), Ok(String::new()));
        }

        #[test]
        fn empty_multiple_times() {
            assert_eq!(repetition("", 2), Ok(String::new()));
        }

        #[test]
        fn nonempty_zero_times() {
            assert_eq!(repetition("ab", 0), Ok(String::new()));
        }

        #[test]
        fn nonempty_multiple_times() {
            assert_eq!(repetition("ab", 2), Ok(String::from("abab")));
        }
    }

    #[test]
    fn counter() {
        use super::*;

        assert_eq!(counter(123), Ok(String::from("123")))
    }

    mod random_number {
        use super::*;

        #[test]
        fn lowest() {
            assert_eq!(random_number(0, Some(0)), Ok(String::from("0")));
        }

        #[test]
        fn highest() {
            assert_eq!(
                random_number(Number::MAX, None),
                Ok(Number::MAX.to_string())
            );
        }

        #[test]
        fn lowest_to_highest() {
            assert_ok!(random_number(0, Some(Number::MAX))); // Should not overflow
            assert_ok!(random_number(1, Some(Number::MAX)));
            assert_ok!(random_number(0, Some(Number::MAX - 1)));
        }
    }

    #[test]
    fn random_uuid() {
        use super::*;
        use crate::pattern::filter::testing::assert_ok_uuid;

        assert_ok_uuid(random_uuid());
    }
}
