use crate::cli::ErrorKind;
use crate::cli::ParseValue;
use crate::cli::ParseValueResult;
use anyhow::anyhow;
use bstr::ByteSlice;
use os_str_bytes::OsStrBytes;
use std::borrow::Cow;
use std::fmt::Display;
use std::num::ParseIntError;
use std::path::Path;

pub fn path_from_io_bytes(bytes: &[u8]) -> Result<&Path, ErrorKind<'static>> {
    match Path::from_io_bytes(bytes) {
        Some(path) => Ok(path),
        None => Err(ErrorKind::RuntimeError(anyhow!(
            "Unable to decode path from stdin: {}",
            bytes.to_str_lossy()
        ))),
    }
}

#[derive(Default, Clone, Debug)]
pub struct ByteSize(pub usize);

impl ParseValue<str> for ByteSize {
    fn parse_value(raw_value: Cow<str>) -> ParseValueResult<String> {
        match parse_byte_size(raw_value.as_ref()) {
            Ok(size) => Ok(Box::new(ByteSize(size))),
            Err(err) => Err((raw_value.into(), err.into())),
        }
    }
}

impl Display for ByteSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (divider, unit) = if self.0 > 1 << 30 {
            (1 << 30, "GiB")
        } else if self.0 > 1 << 20 {
            (1 << 20, "MiB")
        } else if self.0 > 1 << 10 {
            (1 << 10, "KiB")
        } else {
            (1, "B")
        };

        let result = self.0 as f32 / divider as f32;
        let precision = 100.0;
        let rounded_result = (result * precision).floor() / precision;
        write!(f, "{rounded_result} {unit}")
    }
}

fn parse_byte_size(value: &str) -> Result<usize, ParseIntError> {
    let (value, multiplier) = if let Some(value) = strip_any_suffix(value, &["GiB", "GB", "G"]) {
        (value, 1 << 30)
    } else if let Some(value) = strip_any_suffix(value, &["MiB", "MB", "M"]) {
        (value, 1 << 20)
    } else if let Some(value) = strip_any_suffix(value, &["KiB", "KB", "K"]) {
        (value, 1 << 10)
    } else {
        (value.strip_suffix("B").unwrap_or(value), 1)
    };
    match value.trim_end().parse::<usize>() {
        Ok(value) => Ok(value * multiplier),
        Err(err) => Err(err),
    }
}

fn strip_any_suffix<'a>(value: &'a str, suffixes: &[&str]) -> Option<&'a str> {
    for suffix in suffixes {
        if let Some(prefix) = value.strip_suffix(suffix) {
            return Some(prefix);
        }
    }
    None
}

#[cfg(test)]
pub fn strip_colors(value: &str) -> String {
    use anstream::StripStream;
    use bstr::ByteSlice;
    use std::io::Write;

    let mut writer = StripStream::new(Vec::new());
    writer.write_all(value.as_bytes()).unwrap();
    writer.into_inner().to_str_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use crate::utils::ByteSize;
    use claims::*;
    use rstest::rstest;

    #[rstest]
    #[case(123, "123 B")]
    #[case(123 * 1024, "123 KiB")]
    #[case(123 * 1024 * 1024, "123 MiB")]
    #[case(123 * 1024 * 1024* 1024, "123 GiB")]
    #[case((123.456 * 1024.0) as usize, "123.45 KiB")]
    #[case((123.456 * 1024.0 * 1024.0) as usize, "123.45 MiB")]
    #[case((123.456 * 1024.0 * 1024.0 * 1024.0) as usize, "123.45 GiB")]
    fn display_byte_size(#[case] input: usize, #[case] output: &str) {
        assert_eq!(ByteSize(input).to_string(), output);
    }

    #[rstest]
    #[case("123", 123)]
    #[case("123B", 123)]
    #[case("123 B", 123)]
    #[case("123K", 123 *1024)]
    #[case("123KB", 123 *1024)]
    #[case("123KiB", 123 *1024)]
    #[case("123 K", 123 *1024)]
    #[case("123 KB", 123 *1024)]
    #[case("123 KiB", 123 *1024)]
    #[case("123M", 123 *1024 * 1024)]
    #[case("123MB", 123 *1024 * 1024)]
    #[case("123MiB", 123 *1024 * 1024)]
    #[case("123 M", 123 *1024 * 1024)]
    #[case("123 MB", 123 *1024 * 1024)]
    #[case("123 MiB", 123 *1024 * 1024)]
    #[case("123G", 123 * 1024 * 1024 * 1024)]
    #[case("123GB", 123 * 1024 * 1024 * 1024)]
    #[case("123GiB", 123 * 1024 * 1024 * 1024)]
    #[case("123 G", 123 * 1024 * 1024 * 1024)]
    #[case("123 GB", 123 * 1024 * 1024 * 1024)]
    #[case("123 GiB", 123 * 1024 * 1024 * 1024)]
    fn parse_byte_size(#[case] input: &str, #[case] output: usize) {
        assert_ok_eq!(super::parse_byte_size(input), output);
    }
}
