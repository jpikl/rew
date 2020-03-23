use crate::pattern::error::ParseError;
use crate::pattern::number::parse_usize;
use crate::pattern::source::Source;

#[derive(Debug, PartialEq)]
pub enum Variable {
    Filename,
    Basename,
    Extension,
    ExtensionWithDot,
    LocalCounter,
    GlobalCounter,
    RegexGroup(usize),
    Uuid,
}

impl Variable {
    pub fn parse(string: &str) -> Result<Variable, ParseError> {
        let mut source = Source::new(string);

        let variable = match source.peek() {
            Some('0'..='9') => Variable::RegexGroup(parse_usize(&mut source)?),
            Some(ch) => {
                source.consume();
                match ch {
                    'f' => Variable::Filename,
                    'b' => Variable::Basename,
                    'e' => Variable::Extension,
                    'E' => Variable::ExtensionWithDot,
                    'c' => Variable::LocalCounter,
                    'C' => Variable::GlobalCounter,
                    'u' => Variable::Uuid,
                    _ => {
                        return Err(ParseError {
                            message: "Unknown variable",
                            position: 0,
                        });
                    }
                }
            }
            None => {
                return Err(ParseError {
                    message: "Empty input",
                    position: 0,
                })
            }
        };

        if source.peek().is_none() {
            Ok(variable)
        } else {
            Err(ParseError {
                message: "Unexpected character",
                position: source.position(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_as_error() {
        assert_eq!(
            Variable::parse(""),
            Err(ParseError {
                message: "Empty input",
                position: 0,
            })
        )
    }

    #[test]
    fn parse_filename() {
        assert_eq!(Variable::parse("f"), Ok(Variable::Filename));
    }

    #[test]
    fn parse_basename() {
        assert_eq!(Variable::parse("b"), Ok(Variable::Basename));
    }

    #[test]
    fn parse_extension() {
        assert_eq!(Variable::parse("e"), Ok(Variable::Extension));
    }

    #[test]
    fn parse_extension_with_dot() {
        assert_eq!(Variable::parse("E"), Ok(Variable::ExtensionWithDot));
    }

    #[test]
    fn parse_local_counter() {
        assert_eq!(Variable::parse("c"), Ok(Variable::LocalCounter));
    }

    #[test]
    fn parse_global_counter() {
        assert_eq!(Variable::parse("C"), Ok(Variable::GlobalCounter));
    }

    #[test]
    fn parse_regex_group() {
        assert_eq!(Variable::parse("1"), Ok(Variable::RegexGroup(1)));
        assert_eq!(Variable::parse("2"), Ok(Variable::RegexGroup(2)));
        assert_eq!(Variable::parse("3"), Ok(Variable::RegexGroup(3)));
        assert_eq!(Variable::parse("4"), Ok(Variable::RegexGroup(4)));
        assert_eq!(Variable::parse("5"), Ok(Variable::RegexGroup(5)));
        assert_eq!(Variable::parse("6"), Ok(Variable::RegexGroup(6)));
        assert_eq!(Variable::parse("7"), Ok(Variable::RegexGroup(7)));
        assert_eq!(Variable::parse("8"), Ok(Variable::RegexGroup(8)));
        assert_eq!(Variable::parse("9"), Ok(Variable::RegexGroup(9)));
        assert_eq!(Variable::parse("10"), Ok(Variable::RegexGroup(10)));
    }

    #[test]
    fn parse_uuid() {
        assert_eq!(Variable::parse("u"), Ok(Variable::Uuid));
    }

    #[test]
    fn parse_unknown_variable_as_error() {
        assert_eq!(
            Variable::parse("_"),
            Err(ParseError {
                message: "Unknown variable",
                position: 0,
            })
        );
    }

    #[test]
    fn parse_unexpected_character_as_error() {
        assert_eq!(
            Variable::parse("f_"),
            Err(ParseError {
                message: "Unexpected character",
                position: 1,
            })
        );
        assert_eq!(
            Variable::parse("123_"),
            Err(ParseError {
                message: "Unexpected character",
                position: 3,
            })
        );
    }
}
