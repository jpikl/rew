use common::color::spec_bold_color;
use common::io::Output;
#[cfg(test)]
use std::ffi::OsStr;
use std::fmt;
use std::fmt::Debug;
use std::io::Result;
use std::ops::Range;
use termcolor::Color;

#[derive(Debug, Clone)]
pub struct AnyString(pub String);

impl PartialEq for AnyString {
    fn eq(&self, _: &Self) -> bool {
        // This is only useful when comparing system error messages in tests,
        // because we cannot rely on a specific error message.
        true
    }
}

impl fmt::Display for AnyString {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

pub trait HasRange {
    fn range(&self) -> &Range<usize>;
}

pub fn highlight_range<O: Output>(
    output: &mut O,
    string: &str,
    range: &Range<usize>,
    color: Color,
) -> Result<()> {
    write!(output, "{}", &string[..range.start])?;
    output.set_color(&spec_bold_color(color))?;
    write!(output, "{}", &string[range.start..range.end])?;
    output.reset()?;
    writeln!(output, "{}", &string[range.end..])?;

    let spaces_count = string[..range.start].chars().count();
    let markers_count = string[range.start..range.end].chars().count().max(1);

    write!(output, "{}", " ".repeat(spaces_count))?;
    output.set_color(&spec_bold_color(color))?;
    write!(output, "{}", "^".repeat(markers_count))?;
    output.reset()?;

    writeln!(output)
}

#[cfg(all(test, any(unix)))]
pub fn make_non_utf8_os_str<'a>() -> &'a OsStr {
    use std::os::unix::ffi::OsStrExt;
    OsStr::from_bytes(&[0x66, 0x6f, 0x80, 0x6f][..])
}

#[cfg(all(test, any(windows)))]
pub fn make_non_utf8_os_str<'a>() -> &'a OsStr {
    use std::ffi::OsString;
    use std::os::windows::prelude::*;
    OsString::from_wide(&[0x0066, 0x006f, 0xD800, 0x006f][..]).as_os_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::io::mem::{MemoryOutput, OutputChunk};

    #[test]
    fn any_string_eq() {
        assert_eq!(AnyString(String::from("a")), AnyString(String::from("a")));
        assert_eq!(AnyString(String::from("a")), AnyString(String::from("b")));
    }

    #[test]
    fn any_string_fmt() {
        assert_eq!(AnyString(String::from("abc")).to_string(), "abc");
    }

    #[test]
    fn highlights_range() {
        let mut output = MemoryOutput::new();
        highlight_range(&mut output, "abcde", &(1..4), Color::Green).unwrap();

        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::plain("a"),
                OutputChunk::bold_color(Color::Green, "bcd"),
                OutputChunk::plain("e\n "),
                OutputChunk::bold_color(Color::Green, "^^^"),
                OutputChunk::plain("\n")
            ]
        );
    }
}
