use std::ascii::escape_default;
use std::fmt::Write;

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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case('a' => "a"; "ascii")]
    #[test_case('á' => "á"; "non-ascii")]
    #[test_case('\0' => "\\0"; "null")]
    #[test_case('\x01' => "\\x01"; "0x01")]
    #[test_case('\n' => "\\n"; "line feed")]
    #[test_case('\r' => "\\r"; "carriage return")]
    #[test_case('\t' => "\\t"; "horizontal tab")]
    fn escape_char(char: char) -> String {
        super::escape_char(char)
    }

    #[test_case("abc123" => "abc123"; "no escaping")]
    #[test_case("abc\0\0x01\n\r\táčď" => "abc\\0\\0x01\\n\\r\\táčď"; "with escaping")]
    fn escape_str(string: &str) -> String {
        super::escape_str(string)
    }
}
