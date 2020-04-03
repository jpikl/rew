use crate::pattern::number::parse_usize;
use crate::pattern::parse::ParseError;
use crate::pattern::reader::Reader;
use crate::pattern::variable::Variable;

impl Variable {
    pub fn parse(string: &str) -> Result<Self, ParseError> {
        let mut reader = Reader::new(string);

        let variable = match reader.peek() {
            Some('0'..='9') => Variable::CaptureGroup(parse_usize(&mut reader)?),
            Some(ch) => {
                reader.read();
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

        if reader.peek().is_none() {
            Ok(variable)
        } else {
            Err(ParseError {
                message: "Unexpected character",
                position: reader.position(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename() {
        assert_eq!(Variable::parse("f"), Ok(Variable::Filename));
    }

    #[test]
    fn basename() {
        assert_eq!(Variable::parse("b"), Ok(Variable::Basename));
    }

    #[test]
    fn extension() {
        assert_eq!(Variable::parse("e"), Ok(Variable::Extension));
    }

    #[test]
    fn extension_with_dot() {
        assert_eq!(Variable::parse("E"), Ok(Variable::ExtensionWithDot));
    }

    #[test]
    fn local_counter() {
        assert_eq!(Variable::parse("c"), Ok(Variable::LocalCounter));
    }

    #[test]
    fn global_counter() {
        assert_eq!(Variable::parse("C"), Ok(Variable::GlobalCounter));
    }

    #[test]
    fn regex_group() {
        assert_eq!(Variable::parse("1"), Ok(Variable::CaptureGroup(1)));
        assert_eq!(Variable::parse("2"), Ok(Variable::CaptureGroup(2)));
        assert_eq!(Variable::parse("3"), Ok(Variable::CaptureGroup(3)));
        assert_eq!(Variable::parse("4"), Ok(Variable::CaptureGroup(4)));
        assert_eq!(Variable::parse("5"), Ok(Variable::CaptureGroup(5)));
        assert_eq!(Variable::parse("6"), Ok(Variable::CaptureGroup(6)));
        assert_eq!(Variable::parse("7"), Ok(Variable::CaptureGroup(7)));
        assert_eq!(Variable::parse("8"), Ok(Variable::CaptureGroup(8)));
        assert_eq!(Variable::parse("9"), Ok(Variable::CaptureGroup(9)));
        assert_eq!(Variable::parse("10"), Ok(Variable::CaptureGroup(10)));
    }

    #[test]
    fn uuid() {
        assert_eq!(Variable::parse("u"), Ok(Variable::Uuid));
    }

    #[test]
    fn unknown_variable_error() {
        assert_eq!(
            Variable::parse("_"),
            Err(ParseError {
                message: "Unknown variable",
                position: 0,
            })
        );
    }

    #[test]
    fn unexpected_character_error() {
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

    #[test]
    fn empty_error() {
        assert_eq!(
            Variable::parse(""),
            Err(ParseError {
                message: "Empty input",
                position: 0,
            })
        )
    }
}
