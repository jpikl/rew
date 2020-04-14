use crate::pattern::lexer::{Lexer, Token};
use crate::pattern::Pattern;
use std::io;
use termcolor::{Color, ColorSpec, WriteColor};

const CONSTANT_COLOR: Color = Color::Green;
const VARIABLE_COLOR: Color = Color::Blue;
const SYMBOL_COLOR: Color = Color::Magenta;

impl Pattern {
    pub fn render<S: io::Write + WriteColor>(stream: &mut S, string: &str) -> io::Result<()> {
        let mut in_expression = false;
        let mut position = 0;

        for result in Lexer::new(&string) {
            if let Ok(token) = result {
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

                stream.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(bold))?;
                write!(stream, "{}", &string[token.start..token.end])?;
                position = token.end;
            } else {
                break;
            }
        }

        stream.reset()?;
        write!(stream, "{}", &string[position..])
    }
}
