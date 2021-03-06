use std::ascii::escape_default;
use std::fmt::Write;

fn append_escaped_char(string: &mut String, char: char) {
    if char.is_ascii() {
        if char == '\0' {
            string.push_str("\\0"); // We do not want the default '\x00' output
        } else {
            write!(string, "{}", escape_default(char as u8))
                .expect("Failed to append escaped char to string");
        }
    } else {
        string.push(char);
    }
}

pub fn escape_char(char: char) -> String {
    let mut result = String::new();
    append_escaped_char(&mut result, char);
    result
}

pub fn escape_str(str: &str) -> String {
    let mut result = String::new();
    for char in str.chars() {
        append_escaped_char(&mut result, char);
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn escape_char() {
        use super::*;

        assert_eq!(escape_char('a'), String::from("a"));
        assert_eq!(escape_char('á'), String::from("á"));
        assert_eq!(escape_char('\0'), String::from("\\0"));
        assert_eq!(escape_char('\x01'), String::from("\\x01"));
        assert_eq!(escape_char('\n'), String::from("\\n"));
        assert_eq!(escape_char('\r'), String::from("\\r"));
        assert_eq!(escape_char('\t'), String::from("\\t"));
    }

    #[test]
    fn escape_str() {
        use super::*;

        assert_eq!(escape_str("abc123"), String::from("abc123"));
        assert_eq!(
            escape_str("abc\0\0x01\n\r\táčď"),
            String::from("abc\\0\\0x01\\n\\r\\táčď")
        );
    }
}
