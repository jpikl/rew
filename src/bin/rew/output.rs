use crate::utils::HasRange;
use common::color::{spec_bold_color, spec_color};
use common::output::write_error;
use common::symbols::{DIFF_IN, DIFF_OUT};
use std::error::Error;
use std::io::{Result, Write};
use std::ops::Range;
use termcolor::{Color, WriteColor};

pub enum Mode {
    Standard,
    StandardNoTrailingDelimiter,
    Diff,
    Pretty,
}

pub struct Values<O: Write + WriteColor> {
    output: O,
    mode: Mode,
    delimiter: String,
    first_result: bool,
    flush_needed: bool,
}

impl<O: Write + WriteColor> Values<O> {
    pub fn new(output: O, mode: Mode, delimiter: &str) -> Self {
        Self {
            output,
            mode,
            delimiter: delimiter.to_string(),
            first_result: true,
            flush_needed: !delimiter.ends_with('\n'),
        }
    }

    pub fn write(&mut self, input_value: &str, output_value: &str) -> Result<()> {
        match self.mode {
            Mode::Standard => {
                write!(self.output, "{}{}", output_value, self.delimiter)?;
                self.flush_if_needed()
            }
            Mode::StandardNoTrailingDelimiter => {
                if self.first_result {
                    self.first_result = false;
                    write!(self.output, "{}", output_value)
                } else {
                    write!(self.output, "{}", self.delimiter)?;
                    self.flush_if_needed()?;
                    write!(self.output, "{}", output_value)
                }
            }
            Mode::Diff => {
                write!(
                    self.output,
                    "{}{}{}{}{}{}",
                    DIFF_IN, input_value, self.delimiter, DIFF_OUT, output_value, self.delimiter
                )?;
                self.flush_if_needed()
            }
            Mode::Pretty => {
                self.output.set_color(&spec_color(Color::Blue))?;
                write!(self.output, "{}", input_value)?;
                self.output.reset()?;
                write!(self.output, " -> ")?;
                self.output.set_color(&spec_color(Color::Green))?;
                writeln!(self.output, "{}", output_value)
            }
        }
    }

    fn flush_if_needed(&mut self) -> Result<()> {
        if self.flush_needed {
            self.output.flush()
        } else {
            Ok(())
        }
    }
}

pub fn write_pattern_error<O: Write + WriteColor, E: Error + HasRange>(
    output: &mut O,
    error: &E,
    raw_pattern: &str,
) -> Result<()> {
    write_error(output, error)?;
    writeln!(output)?;
    highlight_range(output, raw_pattern, error.range(), Color::Red)?;
    output.reset()
}

pub fn highlight_range<O: Write + WriteColor>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use common::testing::{ColoredOuput, OutputChunk};
    use std::fmt;
    use std::ops::Range;

    #[test]
    fn standard_mode_no_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Standard, "");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("bd")])
    }

    #[test]
    fn standard_mode_newline_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Standard, "\n");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("b\nd\n")])
    }

    #[test]
    fn standard_mode_nul_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Standard, "\0");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("b\0d\0")])
    }

    #[test]
    fn standard_mode_with_ntr_no_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::StandardNoTrailingDelimiter, "");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("bd")])
    }

    #[test]
    fn standard_mode_with_ntr_newline_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::StandardNoTrailingDelimiter, "\n");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("b\nd")])
    }

    #[test]
    fn standard_mode_with_ntr_nul_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::StandardNoTrailingDelimiter, "\0");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("b\0d")])
    }

    #[test]
    fn diff_mode_no_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Diff, "");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("<a>b<c>d")])
    }

    #[test]
    fn diff_mode_newline_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Diff, "\n");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("<a\n>b\n<c\n>d\n")])
    }

    #[test]
    fn diff_mode_null_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Diff, "\0");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("<a\0>b\0<c\0>d\0")])
    }

    #[test]
    fn pretty_mode() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Pretty, "ignored");
        write_values(&mut values);
        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::color(Color::Blue, "a"),
                OutputChunk::plain(" -> "),
                OutputChunk::color(Color::Green, "b\n"),
                OutputChunk::color(Color::Blue, "c"),
                OutputChunk::plain(" -> "),
                OutputChunk::color(Color::Green, "d\n")
            ]
        )
    }

    fn write_values(values: &mut Values<&mut ColoredOuput>) {
        values.write("a", "b").unwrap();
        values.write("c", "d").unwrap();
    }

    #[test]
    fn writes_pattern_error() {
        #[derive(Debug)]
        struct CustomError {}
        impl Error for CustomError {}

        impl fmt::Display for CustomError {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("msg")
            }
        }

        impl HasRange for CustomError {
            fn range(&self) -> &Range<usize> {
                &(1..3)
            }
        }

        let mut output = ColoredOuput::new();
        write_pattern_error(&mut output, &CustomError {}, "abcd").unwrap();

        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::color(Color::Red, "error:"),
                OutputChunk::plain(" msg\n\na"),
                OutputChunk::bold_color(Color::Red, "bc"),
                OutputChunk::plain("d\n "),
                OutputChunk::bold_color(Color::Red, "^^"),
                OutputChunk::plain("\n")
            ]
        );
    }

    #[test]
    fn highlights_range() {
        let mut output = ColoredOuput::new();
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
