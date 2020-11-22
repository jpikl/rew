use crate::output::highlight_range;
use crate::pattern::parse::Parsed;
use crate::pattern::parser::Item;
use crate::pattern::Pattern;
use common::color::spec_color;
use std::fmt::Display;
use std::io::{Result, Write};
use termcolor::{Color, WriteColor};

impl Pattern {
    pub fn explain<O: Write + WriteColor>(&self, output: &mut O) -> Result<()> {
        for item in &self.items {
            match &item.value {
                Item::Constant(_) => self.explain_part(output, &item, Color::Green),
                Item::Expression(filters) => {
                    self.explain_part(output, &item, Color::Yellow)?;
                    for filter in filters {
                        self.explain_part(output, &filter, Color::Blue)?;
                    }
                    Ok(())
                }
            }?;
        }
        Ok(())
    }

    fn explain_part<O: Write + WriteColor, T: Display>(
        &self,
        output: &mut O,
        part: &Parsed<T>,
        color: Color,
    ) -> Result<()> {
        highlight_range(output, &self.source, &part.range, color)?;
        writeln!(output)?;
        output.set_color(&spec_color(color))?;
        write!(output, "{}", part.value)?;
        output.reset()?;
        write!(output, "\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::filter::Filter;
    use crate::pattern::parse::Parsed;
    use common::testing::{ColoredOuput, OutputChunk};

    #[test]
    fn explain_empty() {
        let pattern = Pattern {
            source: String::new(),
            items: Vec::new(),
        };

        let mut output = ColoredOuput::new();
        pattern.explain(&mut output).unwrap();

        assert_eq!(output.chunks(), &[]);
    }

    #[test]
    fn explain_complex() {
        let pattern = Pattern {
            source: String::from("_{f|t}"),
            items: vec![
                Parsed {
                    value: Item::Constant(String::from("_")),
                    range: 0..1,
                },
                Parsed {
                    value: Item::Expression(vec![
                        Parsed {
                            value: Filter::FileName,
                            range: 2..3,
                        },
                        Parsed {
                            value: Filter::Trim,
                            range: 4..5,
                        },
                    ]),
                    range: 1..6,
                },
            ],
        };

        let mut output = ColoredOuput::new();
        pattern.explain(&mut output).unwrap();

        assert_eq!(
            output.chunks(),
            &[
                OutputChunk::bold_color(Color::Green, "_"),
                OutputChunk::plain("{f|t}\n"),
                OutputChunk::bold_color(Color::Green, "^"),
                OutputChunk::plain("\n\n"),
                OutputChunk::color(Color::Green, "Constant '_'"),
                OutputChunk::plain("\n\n_"),
                OutputChunk::bold_color(Color::Yellow, "{f|t}"),
                OutputChunk::plain("\n "),
                OutputChunk::bold_color(Color::Yellow, "^^^^^"),
                OutputChunk::plain("\n\n"),
                OutputChunk::color(Color::Yellow, "Expression with 2 filters"),
                OutputChunk::plain("\n\n_{"),
                OutputChunk::bold_color(Color::Blue, "f"),
                OutputChunk::plain("|t}\n  "),
                OutputChunk::bold_color(Color::Blue, "^"),
                OutputChunk::plain("\n\n"),
                OutputChunk::color(Color::Blue, "File name"),
                OutputChunk::plain("\n\n_{f|"),
                OutputChunk::bold_color(Color::Blue, "t"),
                OutputChunk::plain("}\n    "),
                OutputChunk::bold_color(Color::Blue, "^"),
                OutputChunk::plain("\n\n"),
                OutputChunk::color(Color::Blue, "Trim"),
                OutputChunk::plain("\n\n")
            ]
        );
    }
}
