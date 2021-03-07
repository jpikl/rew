use uuid::Uuid;

pub fn random_uuid() -> String {
    let mut buffer = Uuid::encode_buffer();
    let str = Uuid::new_v4().to_hyphenated().encode_lower(&mut buffer);
    (*str).to_string()
}

#[cfg(test)]
pub fn assert_uuid(value: &str) {
    let regex_str = "^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
    let regex = regex::Regex::new(regex_str).unwrap();
    assert!(regex.is_match(&value), "{} is UUID v4", value);
}

#[cfg(test)]
mod tests {
    #[test]
    fn random_uuid() {
        use super::*;
        assert_uuid(&random_uuid());
    }
}
