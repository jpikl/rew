use std::fmt;
use std::fmt::Debug;
use std::io::{Result, Write};
use std::ops::Range;
use termcolor::{Color, ColorSpec, WriteColor};

#[derive(Debug, Clone)]
pub struct AnyString(pub String);

impl PartialEq for AnyString {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl fmt::Display for AnyString {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

pub fn spec_color(color: Color) -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color));
    spec
}

pub fn spec_bold_color(color: Color) -> ColorSpec {
    let mut spec = spec_color(color);
    spec.set_bold(true);
    spec
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
