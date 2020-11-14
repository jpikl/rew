use crate::utils::{highlight_range, HasRange};
use common::color::spec_color;
use common::output::write_error;
use common::symbols::{DIFF_IN, DIFF_OUT};
use std::error::Error;
use std::io::{Result, Write};
use termcolor::{Color, WriteColor};

pub enum Mode {
    Standard,
    Diff,
    Pretty,
}

pub struct Values<O: Write + WriteColor> {
    output: O,
    mode: Mode,
    delimiter: String,
    flush_needed: bool,
}

impl<O: Write + WriteColor> Values<O> {
    pub fn new(output: O, mode: Mode, delimiter: &str) -> Self {
        Self {
            output,
            mode,
            delimiter: delimiter.to_string(),
            flush_needed: !delimiter.ends_with('\n'),
        }
    }

    pub fn write(&mut self, input_value: &str, output_value: &str) -> Result<()> {
        match self.mode {
            Mode::Standard => {
                write!(self.output, "{}{}", output_value, self.delimiter)?;
                self.flush_if_needed()
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

#[cfg(test)]
mod tests {
    use super::*;
    use common::testing::{ColoredOuput, OutputChunk};
    use std::fmt;
    use std::ops::Range;

    #[test]
    fn values_out_no_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Standard, "");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("bd")])
    }

    #[test]
    fn values_out_newline_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Standard, "\n");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("b\nd\n")])
    }

    #[test]
    fn values_out_nul_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Standard, "\0");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("b\0d\0")])
    }

    #[test]
    fn values_diff_no_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Diff, "");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("<a>b<c>d")])
    }

    #[test]
    fn values_diff_newline_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Diff, "\n");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("<a\n>b\n<c\n>d\n")])
    }

    #[test]
    fn values_diff_null_delimiter() {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, Mode::Diff, "\0");
        write_values(&mut values);
        assert_eq!(output.chunks(), &[OutputChunk::plain("<a\0>b\0<c\0>d\0")])
    }

    #[test]
    fn values_pretty() {
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
}
