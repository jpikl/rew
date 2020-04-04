use crate::pattern::char::Char;
use crate::pattern::error::ErrorType;
use crate::pattern::number::parse_usize;
use crate::pattern::parse::ParseError;
use crate::pattern::reader::Reader;
use crate::pattern::variable::Variable;

impl Variable {
    pub fn parse(chars: Vec<Char>) -> Result<Self, ParseError> {
        let mut reader = Reader::new(chars);
        let position = reader.position();

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
                            typ: ErrorType::UnknownVariable,
                            start: position,
                            end: reader.position(),
                        });
                    }
                }
            }
            None => {
                return Err(ParseError {
                    typ: ErrorType::ExpectedVariable,
                    start: position,
                    end: reader.end(),
                })
            }
        };

        if reader.peek().is_none() {
            Ok(variable)
        } else {
            Err(ParseError {
                typ: ErrorType::UnexpectedCharacters,
                start: reader.position(),
                end: reader.end(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename() {
        assert_ok("f", Variable::Filename);
    }

    #[test]
    fn basename() {
        assert_ok("b", Variable::Basename);
    }

    #[test]
    fn extension() {
        assert_ok("e", Variable::Extension);
    }

    #[test]
    fn extension_with_dot() {
        assert_ok("E", Variable::ExtensionWithDot);
    }

    #[test]
    fn local_counter() {
        assert_ok("c", Variable::LocalCounter);
    }

    #[test]
    fn global_counter() {
        assert_ok("C", Variable::GlobalCounter);
    }

    #[test]
    fn regex_group() {
        assert_ok("1", Variable::CaptureGroup(1));
        assert_ok("2", Variable::CaptureGroup(2));
        assert_ok("3", Variable::CaptureGroup(3));
        assert_ok("4", Variable::CaptureGroup(4));
        assert_ok("5", Variable::CaptureGroup(5));
        assert_ok("6", Variable::CaptureGroup(6));
        assert_ok("7", Variable::CaptureGroup(7));
        assert_ok("8", Variable::CaptureGroup(8));
        assert_ok("9", Variable::CaptureGroup(9));
        assert_ok("10", Variable::CaptureGroup(10));
    }

    #[test]
    fn uuid() {
        assert_ok("u", Variable::Uuid);
    }

    #[test]
    fn unknown_variable_error() {
        assert_err(
            "__",
            ParseError {
                typ: ErrorType::UnknownVariable,
                start: 0,
                end: 1,
            },
        );
    }

    #[test]
    fn unexpected_chars_error() {
        assert_err(
            "f__",
            ParseError {
                typ: ErrorType::UnexpectedCharacters,
                start: 1,
                end: 3,
            },
        );
        assert_err(
            "123__",
            ParseError {
                typ: ErrorType::UnexpectedCharacters,
                start: 3,
                end: 5,
            },
        );
    }

    #[test]
    fn empty_error() {
        assert_err(
            "",
            ParseError {
                typ: ErrorType::ExpectedVariable,
                start: 0,
                end: 0,
            },
        )
    }

    fn assert_ok(string: &str, variable: Variable) {
        assert_eq!(Variable::parse(Char::raw_vec(string)), Ok(variable));
    }

    fn assert_err(string: &str, error: ParseError) {
        assert_eq!(Variable::parse(Char::raw_vec(string)), Err(error));
    }
}
