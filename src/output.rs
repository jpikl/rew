use crate::pattern::{eval, parse, Item, Lexer, Pattern, Token};
use std::fmt;
use std::io::{Result, Write};
use std::ops::Range;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const ERROR_COLOR: Color = Color::Red;
const CONSTANT_COLOR: Color = Color::Green;
const EXPRESSION_COLOR: Color = Color::Yellow;
const VARIABLE_COLOR: Color = Color::Blue;
const FILTER_COLOR: Color = Color::Magenta;
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
        write!(self.stderr, "error:")?;

        self.stderr.reset()?;
        writeln!(self.stderr, " {}\n", message)?;

        let mut lexer = Lexer::from(pattern);
        let mut in_expression = false;
        let mut position = 0;

        while let Ok(Some(token)) = lexer.read_token() {
            let (color, bold) = match token.value {
                Token::Raw(_) => {
                    if in_expression {
                        (EXPRESSION_COLOR, false)
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
            write!(self.stderr, "{}", &pattern[token.range])?;
        }

        self.stderr.reset()?;
        write!(self.stderr, "{}", &pattern[position..])?;

        let spaces_count = pattern[..range.start].chars().count();
        let markers_count = pattern[range.start..range.end].chars().count().max(1);

        write!(self.stderr, "\n{}", " ".repeat(spaces_count))?;
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
        write!(self.stderr, "{}", "^".repeat(markers_count))?;

        self.stderr.reset()?;
        writeln!(self.stderr)
    }

    pub fn write_pattern_explanation(
        &mut self,
        raw_pattern: &str,
        pattern: &Pattern,
    ) -> Result<()> {
        for item in pattern.items() {
            let color = match item.value {
                Item::Constant(_) => CONSTANT_COLOR,
                Item::Expression { .. } => EXPRESSION_COLOR,
            };

            self.write_pattern_item_exlanation(raw_pattern, &item, color)?;

            if let Item::Expression { variable, filters } = &item.value {
                self.write_pattern_item_exlanation(raw_pattern, &variable, VARIABLE_COLOR)?;

                for filter in filters {
                    self.write_pattern_item_exlanation(raw_pattern, &filter, FILTER_COLOR)?;
                }
            }
        }
        Ok(())
    }

    fn write_pattern_item_exlanation<T: fmt::Display>(
        &mut self,
        pattern: &str,
        item: &parse::Output<T>,
        color: Color,
    ) -> Result<()> {
        let parse::Output { value, range } = item;
        let spaces_count = pattern[..range.start].chars().count();
        let markers_count = pattern[range.start..range.end].chars().count().max(1);

        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(color));
        color_spec.set_bold(true);

        write!(self.stdout, "{}", &pattern[..range.start])?;
        self.stdout.set_color(&color_spec)?;
        write!(self.stdout, "{}", &pattern[range.start..range.end])?;
        self.stdout.reset()?;
        write!(
            &mut self.stdout,
            "{}\n{}",
            &pattern[range.end..],
            " ".repeat(spaces_count)
        )?;
        self.stdout.set_color(&color_spec)?;
        writeln!(self.stdout, "{}", "^".repeat(markers_count))?;
        self.stdout.reset()?;

        color_spec.set_bold(false);

        writeln!(self.stdout)?;
        self.stdout.set_color(&color_spec)?;
        writeln!(self.stdout, "{}", value)?;
        self.stdout.reset()?;
        writeln!(self.stdout)
    }

    pub fn write_path(&mut self, path: &str) -> Result<()> {
        if let Some(delimiter_value) = self.delimiter {
            write!(self.stdout, "{}{}", path, delimiter_value)
        } else {
            write!(self.stdout, "{}", path)
        }
    }
}
