use crate::pattern::error::ErrorType;
use crate::pattern::number::parse_usize;
use crate::pattern::parse::ParseError;
use crate::pattern::reader::Reader;
use crate::pattern::variable::Variable;

impl Variable {
    pub fn parse(reader: &mut Reader) -> Result<Self, ParseError> {
        let position = reader.position();

        if let Some('0'..='9') = reader.peek_value() {
            Ok(Variable::CaptureGroup(parse_usize(reader)?))
        } else if let Some(char) = reader.read() {
            match char.value() {
                'f' => Ok(Variable::Filename),
                'b' => Ok(Variable::Basename),
                'e' => Ok(Variable::Extension),
                'E' => Ok(Variable::ExtensionWithDot),
                'c' => Ok(Variable::LocalCounter),
                'C' => Ok(Variable::GlobalCounter),
                'u' => Ok(Variable::Uuid),
                _ => Err(ParseError {
                    typ: ErrorType::UnknownVariable(char.clone()),
                    start: position,
                    end: reader.position(),
                }),
            }
        } else {
            Err(ParseError {
                typ: ErrorType::ExpectedVariable,
                start: position,
                end: reader.end(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::char::Char;

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
    fn ignore_remaning_chars_after_variable() {
        let mut reader = Reader::from("f_");
        Variable::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn ignore_remaning_chars_capture_group_variable() {
        let mut reader = Reader::from("123_");
        Variable::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 3);
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

    #[test]
    fn unknown_variable_error() {
        assert_err(
            "-_",
            ParseError {
                typ: ErrorType::UnknownVariable(Char::Raw('-')),
                start: 0,
                end: 1,
            },
        );
    }

    fn assert_ok(string: &str, variable: Variable) {
        assert_eq!(Variable::parse(&mut Reader::from(string)), Ok(variable));
    }

    fn assert_err(string: &str, error: ParseError) {
        assert_eq!(Variable::parse(&mut Reader::from(string)), Err(error));
    }
}
