use uuid::Uuid;

pub fn random() -> String {
    let mut buffer = Uuid::encode_buffer();
    let str = Uuid::new_v4().to_hyphenated().encode_lower(&mut buffer);
    (*str).to_string()
}

#[cfg(test)]
mod tests {

    use crate::pattern::testing::assert_uuid;

    #[test]
    fn random() {
        use super::*;

        assert_uuid(&random());
    }
}
