use crate::pattern::{eval, parse, Lexer, Token};
use std::fmt;
use std::io::{Result, Write};
use std::ops::Range;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const ERROR_COLOR: Color = Color::Red;
const CONSTANT_COLOR: Color = Color::Green;
const VARIABLE_COLOR: Color = Color::Yellow;
const SYMBOL_COLOR: Color = Color::Blue;

pub struct Output {
    stdout: StandardStream,
    stderr: StandardStream,
    delimiter: Option<char>,
}

impl Output {
    pub fn new(colors: ColorChoice, delimiter: Option<char>) -> Self {
        Self {
            stdout: StandardStream::stdout(colors), // TODO global lock
            stderr: StandardStream::stderr(colors), // TODO global lock
            delimiter,
        }
    }

    pub fn write_parse_error(&mut self, pattern: &str, error: &parse::Error) -> Result<()> {
        self.write_pattern_error(pattern, &error, &error.range)
    }

    pub fn write_eval_error(&mut self, pattern: &str, error: &eval::Error) -> Result<()> {
        self.write_pattern_error(pattern, &error, error.cause.range())
    }

    pub fn write_pattern_error<T: fmt::Display>(
        &mut self,
        pattern: &str,
        message: &T,
        range: &Range<usize>,
    ) -> Result<()> {
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(ERROR_COLOR)))?;
        write!(&mut self.stderr, "error:")?;

        self.stderr.reset()?;
        writeln!(&mut self.stderr, " {}\n", message)?;

        let mut lexer = Lexer::from(pattern);
        let mut in_expression = false;
        let mut position = 0;

        while let Ok(Some(token)) = lexer.read_token() {
            let (color, bold) = match token.value {
                Token::Raw(_) => {
                    if in_expression {
                        (VARIABLE_COLOR, false)
                    } else {
                        (CONSTANT_COLOR, false)
                    }
                }
                Token::ExprStart => {
                    in_expression = true;
                    (SYMBOL_COLOR, true)
                }
                Token::ExprEnd => {
                    in_expression = false;
                    (SYMBOL_COLOR, true)
                }
                Token::Pipe => (SYMBOL_COLOR, true),
            };

            self.stderr
                .set_color(ColorSpec::new().set_fg(Some(color)).set_bold(bold))?;
            position = token.range.end;
            write!(&mut self.stderr, "{}", &pattern[token.range])?;
        }

        self.stderr.reset()?;
        write!(&mut self.stderr, "{}", &pattern[position..])?;

        let spaces_count = pattern[..range.start].chars().count();
        let markers_count = pattern[range.start..range.end].chars().count().max(1);

        write!(&mut self.stderr, "\n{}", " ".repeat(spaces_count))?;
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
        write!(&mut self.stderr, "{}", "^".repeat(markers_count))?;

        self.stderr.reset()?;
        writeln!(&mut self.stderr)
    }

    pub fn write_path(&mut self, path: &str) -> Result<()> {
        if let Some(delimiter_value) = self.delimiter {
            write!(self.stdout, "{}{}", path, delimiter_value)
        } else {
            write!(self.stdout, "{}", path)
        }
    }
}
