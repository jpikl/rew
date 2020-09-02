use common::color::spec_bold_color;
use std::fmt;
use std::fmt::Debug;
use std::io::{Result, Write};
use std::ops::Range;
use termcolor::{Color, WriteColor};

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
        write!(formatter, "{}", self.0)
    }
}

pub trait HasRange {
    fn range(&self) -> &Range<usize>;
}

pub fn highlight_range<S: Write + WriteColor>(
    stream: &mut S,
    string: &str,
    range: &Range<usize>,
    color: Color,
) -> Result<()> {
    write!(stream, "{}", &string[..range.start])?;
    stream.set_color(&spec_bold_color(color))?;
    write!(stream, "{}", &string[range.start..range.end])?;
    stream.reset()?;
    writeln!(stream, "{}", &string[range.end..])?;

    let spaces_count = string[..range.start].chars().count();
    let markers_count = string[range.start..range.end].chars().count().max(1);

    stream.set_color(&spec_bold_color(color))?;
    write!(stream, "{}", " ".repeat(spaces_count))?;
    writeln!(stream, "{}", "^".repeat(markers_count))?;
    stream.reset()
}
