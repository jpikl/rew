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
    let result = if let Some(length) = (end - start).checked_add(1) {
        start + thread_rng().gen_range(0, length)
    } else {
        thread_rng().gen()
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
    use crate::pattern::filter::testing::assert_ok_uuid;
    use claim::*;

    #[test]
    fn generates_repetition() {
        assert_eq!(repetition("a", 0), Ok(String::new()));
        assert_eq!(repetition("", 1), Ok(String::new()));
        assert_eq!(repetition("ab", 2), Ok(String::from("abab")));
    }

    #[test]
    fn generates_counter() {
        assert_eq!(counter(123), Ok(String::from("123")))
    }

    #[test]
    fn generates_random_number() {
        assert_eq!(random_number(0, Some(0)), Ok(String::from("0")));
        assert_eq!(
            random_number(Number::MAX, None),
            Ok(Number::MAX.to_string())
        );
        assert_ok!(random_number(0, None)); // Should not overflow
    }

    #[test]
    fn generates_random_uuid() {
        assert_ok_uuid(random_uuid());
    }
}
