use crate::pattern::filter::error::Result;
use uuid::Uuid;

pub fn uuid() -> Result {
    let mut buffer = Uuid::encode_buffer();
    let str = Uuid::new_v4().to_hyphenated().encode_lower(&mut buffer);
    Ok((*str).to_string())
}

pub fn counter(value: u32) -> Result {
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::filter::testing::assert_ok_uuid;

    #[test]
    fn generates_uuid() {
        assert_ok_uuid(uuid());
    }

    #[test]
    fn generates_counter() {
        assert_eq!(counter(123), Ok(String::from("123")))
    }
}
