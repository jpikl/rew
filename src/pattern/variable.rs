use crate::pattern::error::ParseError;
use crate::pattern::number::parse_usize;
use crate::pattern::source::Source;

#[derive(Debug, PartialEq)]
pub enum Variable {
    Filename,
    Basename,
    Extension,
    LocalCounter,
    GlobalCounter,
    RegexGroup(usize),
    Uuid,
}

pub fn parse_variable(string: &str) -> Result<Variable, ParseError> {
    let mut source = Source::new(string);

    let variable = match source.peek() {
        Some('0'..='9') => match parse_usize(&mut source) {
            Ok(number) => Variable::RegexGroup(number),
            Err(error) => return Err(error),
        },
        Some(ch) => {
            source.consume();
            match ch {
                'f' => Variable::Filename,
                'b' => Variable::Basename,
                'e' => Variable::Extension,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_as_error() {
        assert_eq!(
            parse_variable(""),
            Err(ParseError {
                message: "Empty input",
                position: 0,
            })
        )
    }

    #[test]
    fn parse_filename() {
        assert_eq!(parse_variable("f"), Ok(Variable::Filename));
    }

    #[test]
    fn parse_basename() {
        assert_eq!(parse_variable("b"), Ok(Variable::Basename));
    }

    #[test]
    fn parse_extension() {
        assert_eq!(parse_variable("e"), Ok(Variable::Extension));
    }

    #[test]
    fn parse_local_counter() {
        assert_eq!(parse_variable("c"), Ok(Variable::LocalCounter));
    }

    #[test]
    fn parse_global_counter() {
        assert_eq!(parse_variable("C"), Ok(Variable::GlobalCounter));
    }

    #[test]
    fn parse_regex_group() {
        assert_eq!(parse_variable("1"), Ok(Variable::RegexGroup(1)));
        assert_eq!(parse_variable("2"), Ok(Variable::RegexGroup(2)));
        assert_eq!(parse_variable("3"), Ok(Variable::RegexGroup(3)));
        assert_eq!(parse_variable("4"), Ok(Variable::RegexGroup(4)));
        assert_eq!(parse_variable("5"), Ok(Variable::RegexGroup(5)));
        assert_eq!(parse_variable("6"), Ok(Variable::RegexGroup(6)));
        assert_eq!(parse_variable("7"), Ok(Variable::RegexGroup(7)));
        assert_eq!(parse_variable("8"), Ok(Variable::RegexGroup(8)));
        assert_eq!(parse_variable("9"), Ok(Variable::RegexGroup(9)));
        assert_eq!(parse_variable("10"), Ok(Variable::RegexGroup(10)));
    }

    #[test]
    fn parse_uuid() {
        assert_eq!(parse_variable("u"), Ok(Variable::Uuid));
    }

    #[test]
    fn parse_unknown_variable_as_error() {
        assert_eq!(
            parse_variable("_"),
            Err(ParseError {
                message: "Unknown variable",
                position: 0,
            })
        );
    }

    #[test]
    fn parse_unexpected_character_as_error() {
        assert_eq!(
            parse_variable("f_"),
            Err(ParseError {
                message: "Unexpected character",
                position: 1,
            })
        );
        assert_eq!(
            parse_variable("123_"),
            Err(ParseError {
                message: "Unexpected character",
                position: 3,
            })
        );
    }
}
