use common::color::spec_bold_color;
use common::io::Output;
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

    output.set_color(&spec_bold_color(color))?;
    write!(output, "{}", " ".repeat(spaces_count))?;
    write!(output, "{}", "^".repeat(markers_count))?;
    output.reset()?;

    writeln!(output)
}
