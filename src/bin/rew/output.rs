use crate::utils::{GetIndexRange, IndexRange};
use common::color::{spec_bold_color, spec_color};
use common::output::write_error;
use common::symbols::{DIFF_IN, DIFF_OUT};
use std::error::Error;
use std::io::{Result, Write};
use termcolor::{Color, WriteColor};

pub enum Mode {
    Standard,
    StandardNoEnd,
    Diff,
    Pretty,
    JsonLines,
}

pub struct Values<O: Write + WriteColor> {
    output: O,
    mode: Mode,
    terminator: String,
    first_result: bool,
    flush_needed: bool,
}

impl<O: Write + WriteColor> Values<O> {
    pub fn new(output: O, mode: Mode, terminator: &str) -> Self {
        Self {
            output,
            mode,
            terminator: terminator.into(),
            first_result: true,
            flush_needed: !terminator.ends_with('\n'),
        }
    }

    pub fn write(&mut self, input_value: &str, output_value: &str) -> Result<()> {
        match self.mode {
            Mode::Standard => {
                write!(self.output, "{}{}", output_value, self.terminator)?;
                self.flush_if_needed()
            }
            Mode::StandardNoEnd => {
                if self.first_result {
                    self.first_result = false;
                } else {
                    write!(self.output, "{}", self.terminator)?;
                    self.flush_if_needed()?;
                }
                write!(self.output, "{}", output_value)
            }
            Mode::Diff => {
                write!(
                    self.output,
                    "{}{}{}{}{}{}",
                    DIFF_IN, input_value, self.terminator, DIFF_OUT, output_value, self.terminator
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
            Mode::JsonLines => {
                writeln!(
                    self.output,
                    r#"{{"in":"{}","out":"{}"}}"#,
                    input_value, output_value
                )
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

pub fn write_pattern_error<O: Write + WriteColor, E: Error + GetIndexRange>(
    output: &mut O,
    error: &E,
    raw_pattern: &str,
) -> Result<()> {
    write_error(output, error)?;
    writeln!(output)?;
    highlight_range(output, raw_pattern, error.index_range(), Color::Red)?;
    output.reset()
}

pub fn highlight_range<O: Write + WriteColor>(
    output: &mut O,
    string: &str,
    range: &IndexRange,
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
    use indoc::indoc;
    use test_case::test_case;

    #[test_case(Mode::Standard,      "",   plain("bd")               ; "standard no terminator")]
    #[test_case(Mode::Standard,      "\n", plain("b\nd\n")           ; "standard newline terminator")]
    #[test_case(Mode::Standard,      "\0", plain("b\0d\0")           ; "standard null terminator")]
    #[test_case(Mode::StandardNoEnd, "",   plain("bd")               ; "standard no end no terminator")]
    #[test_case(Mode::StandardNoEnd, "\n", plain("b\nd")             ; "standard no end newline terminator")]
    #[test_case(Mode::StandardNoEnd, "\0", plain("b\0d")             ; "standard no end null terminator")]
    #[test_case(Mode::Diff,          "",   plain("<a>b<c>d")         ; "diff no terminator")]
    #[test_case(Mode::Diff,          "\n", plain("<a\n>b\n<c\n>d\n") ; "diff newline terminator")]
    #[test_case(Mode::Diff,          "\0", plain("<a\0>b\0<c\0>d\0") ; "diff null terminator")]
    #[test_case(Mode::Pretty,        "x",  pretty()                  ; "pretty ")]
    #[test_case(Mode::JsonLines,     "x",  plain(indoc! {r#"
                                               {"in":"a","out":"b"}
                                               {"in":"c","out":"d"}
                                           "#})                      ; "json lines")]
    fn values_write(mode: Mode, terminator: &str, chunks: Vec<OutputChunk>) {
        let mut output = ColoredOuput::new();
        let mut values = Values::new(&mut output, mode, terminator);
        values.write("a", "b").unwrap();
        values.write("c", "d").unwrap();
        assert_eq!(output.chunks(), &chunks);
    }

    pub fn plain(value: &str) -> Vec<OutputChunk> {
        vec![OutputChunk::plain(value)]
    }

    fn pretty() -> Vec<OutputChunk> {
        vec![
            OutputChunk::color(Color::Blue, "a"),
            OutputChunk::plain(" -> "),
            OutputChunk::color(Color::Green, "b\n"),
            OutputChunk::color(Color::Blue, "c"),
            OutputChunk::plain(" -> "),
            OutputChunk::color(Color::Green, "d\n"),
        ]
    }

    #[test]
    fn write_pattern_error() {
        use super::*;
        use std::fmt;

        #[derive(Debug)]
        struct CustomError {}
        impl Error for CustomError {}

        impl fmt::Display for CustomError {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("msg")
            }
        }

        impl GetIndexRange for CustomError {
            fn index_range(&self) -> &IndexRange {
                &(1..3)
            }
        }

        let mut output = ColoredOuput::new();
        super::write_pattern_error(&mut output, &CustomError {}, "abcd").unwrap();

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
    fn highlight_range() {
        let mut output = ColoredOuput::new();
        super::highlight_range(&mut output, "abcde", &(1..4), Color::Green).unwrap();

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
