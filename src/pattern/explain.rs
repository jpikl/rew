use crate::pattern::parse::Output;
use crate::pattern::parser::Item;
use crate::pattern::Pattern;
use crate::utils::{highlight_range, spec_color};
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
        stream.set_color(&spec_color(color))?;
        writeln!(stream, "\n{}\n", part.value)?;
        stream.reset()
    }
}
