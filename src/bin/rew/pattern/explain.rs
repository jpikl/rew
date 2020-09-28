use crate::pattern::parse::Output;
use crate::pattern::parser::Item;
use crate::pattern::Pattern;
use crate::utils::highlight_range;
use common::color::spec_color;
use std::fmt::Display;
use std::io::{Result, Write};
use termcolor::{Color, WriteColor};

impl Pattern {
    pub fn explain<S: Write + WriteColor>(&self, stream: &mut S) -> Result<()> {
        for item in &self.items {
            match &item.value {
                Item::Constant(_) => self.explain_part(stream, &item, Color::Green),
                Item::Expression { variable, filters } => {
                    self.explain_part(stream, &item, Color::Yellow)?;
                    self.explain_part(stream, &variable, Color::Blue)?;
                    for filter in filters {
                        self.explain_part(stream, &filter, Color::Magenta)?;
                    }
                    Ok(())
                }
            }?;
        }
        Ok(())
    }

    fn explain_part<S: Write + WriteColor, T: Display>(
        &self,
        stream: &mut S,
        part: &Output<T>,
        color: Color,
    ) -> Result<()> {
        highlight_range(stream, &self.source, &part.range, color)?;
        writeln!(stream)?;
        stream.set_color(&spec_color(color))?;
        write!(stream, "{}", part.value)?;
        stream.reset()?;
        writeln!(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::filter::Filter;
    use crate::pattern::parse::Output;
    use crate::pattern::variable::Variable;
    use common::output_test::{ColoredBuffer, ColoredChunk};

    #[test]
    fn explain_empty() {
        let pattern = Pattern {
            source: String::new(),
            items: Vec::new(),
        };

        let mut buffer = ColoredBuffer::new();
        pattern.explain(&mut buffer).unwrap();

        assert_eq!(buffer.chunks(), vec![]);
    }

    #[test]
    fn explain_complex() {
        let pattern = Pattern {
            source: String::from("_{f|t|u}"),
            items: vec![
                Output {
                    value: Item::Constant(String::from("_")),
                    range: 0..1,
                },
                Output {
                    value: Item::Expression {
                        variable: Output {
                            value: Variable::FileName,
                            range: 2..3,
                        },
                        filters: vec![
                            Output {
                                value: Filter::Trim,
                                range: 4..5,
                            },
                            Output {
                                value: Filter::ToUppercase,
                                range: 6..7,
                            },
                        ],
                    },
                    range: 1..8,
                },
            ],
        };

        let mut buffer = ColoredBuffer::new();
        pattern.explain(&mut buffer).unwrap();

        assert_eq!(
            buffer.chunks(),
            vec![
                ColoredChunk::bold_color(Color::Green, "_"),
                ColoredChunk::plain("{f|t|u}\n"),
                ColoredChunk::bold_color(Color::Green, "^"),
                ColoredChunk::plain("\n\n"),
                ColoredChunk::color(Color::Green, "Constant '_'"),
                ColoredChunk::plain("\n_"),
                ColoredChunk::bold_color(Color::Yellow, "{f|t|u}"),
                ColoredChunk::plain("\n"),
                ColoredChunk::bold_color(Color::Yellow, " ^^^^^^^"),
                ColoredChunk::plain("\n\n"),
                ColoredChunk::color(Color::Yellow, "Expression with a variable and 2 filters"),
                ColoredChunk::plain("\n_{"),
                ColoredChunk::bold_color(Color::Blue, "f"),
                ColoredChunk::plain("|t|u}\n"),
                ColoredChunk::bold_color(Color::Blue, "  ^"),
                ColoredChunk::plain("\n\n"),
                ColoredChunk::color(Color::Blue, "File name"),
                ColoredChunk::plain("\n_{f|"),
                ColoredChunk::bold_color(Color::Magenta, "t"),
                ColoredChunk::plain("|u}\n"),
                ColoredChunk::bold_color(Color::Magenta, "    ^"),
                ColoredChunk::plain("\n\n"),
                ColoredChunk::color(Color::Magenta, "Trim"),
                ColoredChunk::plain("\n_{f|t|"),
                ColoredChunk::bold_color(Color::Magenta, "u"),
                ColoredChunk::plain("}\n"),
                ColoredChunk::bold_color(Color::Magenta, "      ^"),
                ColoredChunk::plain("\n\n"),
                ColoredChunk::color(Color::Magenta, "To uppercase"),
                ColoredChunk::plain("\n")
            ]
        );
    }
}
