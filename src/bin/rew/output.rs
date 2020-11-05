use crate::utils::{highlight_range, HasRange};
use common::color::spec_color;
use common::output::write_error;
use common::symbols::{DIFF_IN, DIFF_OUT};
use std::error::Error;
use std::io::{Result, Write};
use std::path::Path;
use termcolor::{Color, WriteColor};

pub enum PathMode {
    Out(Option<char>),
    Diff(Option<char>),
    Pretty,
}

pub struct Paths<O: Write + WriteColor> {
    output: O,
    mode: PathMode,
}

impl<O: Write + WriteColor> Paths<O> {
    pub fn new(output: O, mode: PathMode) -> Self {
        Self { output, mode }
    }

    pub fn write(&mut self, input_path: &Path, output_path: &str) -> Result<()> {
        match self.mode {
            PathMode::Out(Some(delimiter)) => {
                write!(self.output, "{}{}", output_path, delimiter)?;
                self.flush_if_needed(delimiter)
            }
            PathMode::Out(None) => {
                write!(self.output, "{}", output_path)?;
                self.output.flush()
            }
            PathMode::Diff(Some(delimiter)) => {
                write!(
                    self.output,
                    "{}{}{}{}{}{}",
                    DIFF_IN,
                    input_path.to_string_lossy(),
                    delimiter,
                    DIFF_OUT,
                    output_path,
                    delimiter
                )?;
                self.flush_if_needed(delimiter)
            }
            PathMode::Diff(None) => {
                write!(
                    self.output,
                    "{}{}{}{}",
                    DIFF_IN,
                    input_path.to_string_lossy(),
                    DIFF_OUT,
                    output_path
                )?;
                self.output.flush()
            }
            PathMode::Pretty => {
                self.output.set_color(&spec_color(Color::Blue))?;
                write!(self.output, "{}", input_path.to_string_lossy())?;
                self.output.reset()?;
                write!(self.output, " -> ")?;
                self.output.set_color(&spec_color(Color::Green))?;
                writeln!(self.output, "{}", output_path)
            }
        }
    }

    fn flush_if_needed(&mut self, delimiter: char) -> Result<()> {
        if delimiter != '\n' {
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
    fn paths_out_no_delimiter() {
        let mut output = ColoredOuput::new();
        let mut paths = Paths::new(&mut output, PathMode::Out(None));
        write_paths(&mut paths);
        assert_eq!(output.chunks(), &[OutputChunk::plain("bd")])
    }

    #[test]
    fn paths_out_newline_delimiter() {
        let mut output = ColoredOuput::new();
        let mut paths = Paths::new(&mut output, PathMode::Out(Some('\n')));
        write_paths(&mut paths);
        assert_eq!(output.chunks(), &[OutputChunk::plain("b\nd\n")])
    }

    #[test]
    fn paths_out_nul_delimiter() {
        let mut output = ColoredOuput::new();
        let mut paths = Paths::new(&mut output, PathMode::Out(Some('\0')));
        write_paths(&mut paths);
        assert_eq!(output.chunks(), &[OutputChunk::plain("b\0d\0")])
    }

    #[test]
    fn paths_diff_no_delimiter() {
        let mut output = ColoredOuput::new();
        let mut paths = Paths::new(&mut output, PathMode::Diff(None));
        write_paths(&mut paths);
        assert_eq!(output.chunks(), &[OutputChunk::plain("<a>b<c>d")])
    }

    #[test]
    fn paths_diff_newline_delimiter() {
        let mut output = ColoredOuput::new();
        let mut paths = Paths::new(&mut output, PathMode::Diff(Some('\n')));
        write_paths(&mut paths);
        assert_eq!(output.chunks(), &[OutputChunk::plain("<a\n>b\n<c\n>d\n")])
    }

    #[test]
    fn paths_diff_null_delimiter() {
        let mut output = ColoredOuput::new();
        let mut paths = Paths::new(&mut output, PathMode::Diff(Some('\0')));
        write_paths(&mut paths);
        assert_eq!(output.chunks(), &[OutputChunk::plain("<a\0>b\0<c\0>d\0")])
    }

    #[test]
    fn paths_pretty() {
        let mut output = ColoredOuput::new();
        let mut paths = Paths::new(&mut output, PathMode::Pretty);
        write_paths(&mut paths);
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

    fn write_paths(paths: &mut Paths<&mut ColoredOuput>) {
        paths.write(&Path::new("a"), "b").unwrap();
        paths.write(&Path::new("c"), "d").unwrap();
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
