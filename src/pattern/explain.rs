use crate::pattern::parse::Output;
use crate::pattern::parser::Item;
use crate::pattern::Pattern;
use crate::utils::{highlight_range, spec_color};
use std::fmt::Display;
use std::io::{Result, Write};
use termcolor::{Color, WriteColor};

impl Pattern {
    pub fn explain<S: Write + WriteColor>(&self, stream: &mut S, raw_pattern: &str) -> Result<()> {
        // TODO add Pattern visitor for Output<T>
        for item in self.items.iter() {
            let color = match item.value {
                Item::Constant(_) => Color::Green,
                Item::Expression { .. } => Color::Yellow,
            };

            self.explain_part(stream, &item, raw_pattern, color)?;

            if let Item::Expression { variable, filters } = &item.value {
                self.explain_part(stream, &variable, raw_pattern, Color::Blue)?;

                for filter in filters {
                    self.explain_part(stream, &filter, raw_pattern, Color::Magenta)?;
                }
            }
        }
        Ok(())
    }

    fn explain_part<S: Write + WriteColor, T: Display>(
        &self,
        stream: &mut S,
        part: &Output<T>,
        raw_pattern: &str,
        color: Color,
    ) -> Result<()> {
        highlight_range(stream, raw_pattern, &part.range, color)?;
        stream.set_color(&spec_color(color))?;
        writeln!(stream, "\n{}\n", part.value)?;
        stream.reset()
    }
}
