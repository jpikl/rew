use std::fmt::Display;
use std::io::{Result, Write};

use common::color::spec_color;
use termcolor::{Color, WriteColor};

use crate::output::highlight_range;
use crate::pattern::parse::Parsed;
use crate::pattern::parser::Item;
use crate::pattern::Pattern;

impl Pattern {
    pub fn explain<O: Write + WriteColor>(&self, output: &mut O, all: bool) -> Result<()> {
        for item in &self.items {
            match &item.value {
                Item::Constant(_) => {
                    if all {
                        self.explain_part(output, item, Color::Green)?;
                    }
                }
                Item::Expression(filters) => {
                    if all {
                        self.explain_part(output, item, Color::Yellow)?;
                    }
                    for filter in filters {
                        self.explain_part(output, filter, Color::Blue)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn explain_part<O, T>(&self, output: &mut O, part: &Parsed<T>, color: Color) -> Result<()>
    where
        O: Write + WriteColor,
        T: Display,
    {
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
    use common::testing::{ColoredOuput, OutputChunk};
    use test_case::test_case;

    use super::*;
    use crate::pattern::filter::Filter;
    use crate::pattern::parse::Parsed;

    #[test_case(empty_pattern(),    false, Vec::new()      ; "empty filters")]
    #[test_case(empty_pattern(),    true,  Vec::new()      ; "empty all")]
    #[test_case(nonempty_pattern(), false, filter_chunks() ; "nonempty filters")]
    #[test_case(nonempty_pattern(), true,  all_chunks()    ; "nonempty all")]
    fn explain(pattern: Pattern, all: bool, chunks: Vec<OutputChunk>) {
        let mut output = ColoredOuput::new();
        pattern.explain(&mut output, all).unwrap();
        assert_eq!(output.chunks(), &chunks);
    }

    fn empty_pattern() -> Pattern {
        Pattern {
            source: String::new(),
            items: Vec::new(),
        }
    }

    fn nonempty_pattern() -> Pattern {
        Pattern {
            source: "_{f|t}".into(),
            items: vec![
                Parsed {
                    value: Item::Constant("_".into()),
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
        }
    }

    fn all_chunks() -> Vec<OutputChunk> {
        vec![
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
            OutputChunk::plain("\n\n"),
        ]
    }

    fn filter_chunks() -> Vec<OutputChunk> {
        vec![
            OutputChunk::plain("_{"),
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
            OutputChunk::plain("\n\n"),
        ]
    }
}
