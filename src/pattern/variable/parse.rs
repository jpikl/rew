use crate::pattern::error::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::number::parse_usize;
use crate::pattern::reader::Reader;
use crate::pattern::variable::Variable;

impl Variable {
    pub fn parse(reader: &mut Reader) -> ParseResult<Self> {
        let position = reader.position();

        if let Some('0'..='9') = reader.peek_value() {
            let number = parse_usize(reader)?;
            if number > 0 {
                Ok(Variable::RegexCapture(number))
            } else {
                Err(ParseError {
                    kind: ParseErrorKind::RegexCaptureZero,
                    start: position,
                    end: reader.position(),
                })
            }
        } else if let Some(char) = reader.read() {
            match char.value() {
                'f' => Ok(Variable::Filename),
                'b' => Ok(Variable::Basename),
                'e' => Ok(Variable::Extension),
                'E' => Ok(Variable::ExtensionWithDot),
                'd' => Ok(Variable::FullDirname),
                'D' => Ok(Variable::ParentDirname),
                'p' => Ok(Variable::FullPath),
                'c' => Ok(Variable::LocalCounter),
                'C' => Ok(Variable::GlobalCounter),
                'u' => Ok(Variable::Uuid),
                _ => Err(ParseError {
                    kind: ParseErrorKind::UnknownVariable(char.clone()),
                    start: position,
                    end: reader.position(),
                }),
            }
        } else {
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
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
    fn full_dirname() {
        assert_ok("d", Variable::FullDirname);
    }

    #[test]
    fn parent_dirname() {
        assert_ok("D", Variable::ParentDirname);
    }

    #[test]
    fn full_path() {
        assert_ok("p", Variable::FullPath);
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
    fn regex_capture() {
        assert_ok("1", Variable::RegexCapture(1));
        assert_ok("2", Variable::RegexCapture(2));
        assert_ok("3", Variable::RegexCapture(3));
        assert_ok("4", Variable::RegexCapture(4));
        assert_ok("5", Variable::RegexCapture(5));
        assert_ok("6", Variable::RegexCapture(6));
        assert_ok("7", Variable::RegexCapture(7));
        assert_ok("8", Variable::RegexCapture(8));
        assert_ok("9", Variable::RegexCapture(9));
        assert_ok("10", Variable::RegexCapture(10));
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
                kind: ParseErrorKind::ExpectedVariable,
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
                kind: ParseErrorKind::UnknownVariable(Char::Raw('-')),
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
