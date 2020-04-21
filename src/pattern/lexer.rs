use crate::pattern::char::Char;
use crate::pattern::error::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::reader::Reader;

pub const DEFAULT_ESCAPE: char = '#';
pub const META_CHARS: [char; 3] = [EXPR_START, PIPE, EXPR_END];

pub const EXPR_START: char = '{';
pub const EXPR_END: char = '}';
pub const PIPE: char = '|';

const LF: char = 'n';
const CR: char = 'r';
const TAB: char = 't';
const NUL: char = '0';

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    pub value: T,
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
impl<T> Parsed<T> {
    pub fn dummy(value: T) -> Self {
        Self {
            value,
            start: 0,
            end: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Raw(Vec<Char>),
    ExprStart,
    ExprEnd,
    Pipe,
}

pub struct Lexer {
    reader: Reader,
    escape: char,
}

impl From<&str> for Lexer {
    fn from(string: &str) -> Self {
        Self::new(Reader::from(string))
    }
}

impl Lexer {
    pub fn new(reader: Reader) -> Self {
        Self {
            reader,
            escape: DEFAULT_ESCAPE,
        }
    }

    pub fn set_escape(&mut self, escape: char) {
        self.escape = escape;
    }

    pub fn read_token(&mut self) -> ParseResult<Option<Parsed<Token>>> {
        let start = self.reader.position();
        let value = match self.reader.peek_value() {
            Some(EXPR_START) => {
                self.reader.seek();
                Token::ExprStart
            }
            Some(EXPR_END) => {
                self.reader.seek();
                Token::ExprEnd
            }
            Some(PIPE) => {
                self.reader.seek();
                Token::Pipe
            }
            Some(_) => match self.read_chars() {
                Ok(chars) => Token::Raw(chars),
                Err(error) => return Err(error),
            },
            None => return Ok(None),
        };
        let end = self.reader.position();
        Ok(Some(Parsed { value, start, end }))
    }

    fn read_chars(&mut self) -> ParseResult<Vec<Char>> {
        let mut chars = Vec::new();

        loop {
            match self.reader.peek_value() {
                Some(EXPR_START) | Some(EXPR_END) | Some(PIPE) | None => break,
                Some(value) if value == self.escape => {
                    let start = self.reader.position();
                    self.reader.seek();
                    match self.read_escaped_char() {
                        Ok(char) => chars.push(char),
                        Err(kind) => {
                            let end = self.reader.position();
                            return Err(ParseError { kind, start, end });
                        }
                    }
                }
                Some(value) => {
                    chars.push(Char::Raw(value));
                    self.reader.seek();
                }
            }
        }

        Ok(chars)
    }

    fn read_escaped_char(&mut self) -> Result<Char, ParseErrorKind> {
        if let Some(value) = self.reader.read_value() {
            let escape_sequence = [self.escape, value];
            let escaped_value = match value {
                EXPR_START => EXPR_START,
                EXPR_END => EXPR_END,
                PIPE => PIPE,
                LF => '\n',
                CR => '\r',
                TAB => '\t',
                NUL => '\0',
                _ if value == self.escape => value,
                _ => return Err(ParseErrorKind::UnknownEscapeSequence(escape_sequence)),
            };
            Ok(Char::Escaped(escaped_value, escape_sequence))
        } else {
            Err(ParseErrorKind::UnterminatedEscapeSequence(self.escape))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        let mut lexer = Lexer::from("");
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn raw_char() {
        let mut lexer = Lexer::from("a");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(Char::raw_vec("a")),
                start: 0,
                end: 1,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn raw_chars() {
        let mut lexer = Lexer::from("abc");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(Char::raw_vec("abc")),
                start: 0,
                end: 3,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn expr_start() {
        let mut lexer = Lexer::from("{");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                start: 0,
                end: 1,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn expr_end() {
        let mut lexer = Lexer::from("}");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                start: 0,
                end: 1,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn pipe() {
        let mut lexer = Lexer::from("|");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                start: 0,
                end: 1,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_expr_start() {
        let mut lexer = Lexer::from("#{");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('{', ['#', '{'])]),
                start: 0,
                end: 2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_expr_end() {
        let mut lexer = Lexer::from("#}");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('}', ['#', '}'])]),
                start: 0,
                end: 2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_pipe() {
        let mut lexer = Lexer::from("#|");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('|', ['#', '|'])]),
                start: 0,
                end: 2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_lf() {
        let mut lexer = Lexer::from("#n");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('\n', ['#', 'n'])]),
                start: 0,
                end: 2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_cr() {
        let mut lexer = Lexer::from("#r");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('\r', ['#', 'r'])]),
                start: 0,
                end: 2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_tab() {
        let mut lexer = Lexer::from("#t");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('\t', ['#', 't'])]),
                start: 0,
                end: 2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_nul() {
        let mut lexer = Lexer::from("#0");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('\0', ['#', '0'])]),
                start: 0,
                end: 2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_escape() {
        let mut lexer = Lexer::from("##");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('#', ['#', '#'])]),
                start: 0,
                end: 2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn custom_escape() {
        let mut lexer = Lexer::from(r"\|");
        lexer.set_escape('\\');
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('|', ['\\', '|'])]),
                start: 0,
                end: 2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn unterminated_escape_error() {
        let mut lexer = Lexer::from("#");
        assert_eq!(
            lexer.read_token(),
            Err(ParseError {
                kind: ParseErrorKind::UnterminatedEscapeSequence('#'),
                start: 0,
                end: 1,
            })
        );
    }

    #[test]
    fn unknown_escape_error() {
        let mut lexer = Lexer::from("#x");
        assert_eq!(
            lexer.read_token(),
            Err(ParseError {
                kind: ParseErrorKind::UnknownEscapeSequence(['#', 'x']),
                start: 0,
                end: 2,
            })
        );
    }

    #[test]
    fn various_tokens() {
        let mut lexer = Lexer::from("a{|}bc{de|fg}hi");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(Char::raw_vec("a")),
                start: 0,
                end: 1,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                start: 1,
                end: 2,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                start: 2,
                end: 3,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                start: 3,
                end: 4,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(Char::raw_vec("bc")),
                start: 4,
                end: 6,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                start: 6,
                end: 7,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(Char::raw_vec("de")),
                start: 7,
                end: 9,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                start: 9,
                end: 10,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(Char::raw_vec("fg")),
                start: 10,
                end: 12,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                start: 12,
                end: 13,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(Char::raw_vec("hi")),
                start: 13,
                end: 15,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn various_tokens_and_escapes() {
        let mut lexer = Lexer::from("a{|}bc#{de#|fg#}hi#n#r#t#0##");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(Char::raw_vec("a")),
                start: 0,
                end: 1,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                start: 1,
                end: 2,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                start: 2,
                end: 3,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                start: 3,
                end: 4,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![
                    Char::Raw('b'),
                    Char::Raw('c'),
                    Char::Escaped('{', ['#', '{']),
                    Char::Raw('d'),
                    Char::Raw('e'),
                    Char::Escaped('|', ['#', '|']),
                    Char::Raw('f'),
                    Char::Raw('g'),
                    Char::Escaped('}', ['#', '}']),
                    Char::Raw('h'),
                    Char::Raw('i'),
                    Char::Escaped('\n', ['#', 'n']),
                    Char::Escaped('\r', ['#', 'r']),
                    Char::Escaped('\t', ['#', 't']),
                    Char::Escaped('\0', ['#', '0']),
                    Char::Escaped('#', ['#', '#']),
                ]),
                start: 4,
                end: 28,
            }))
        );
    }

    #[test]
    fn various_tokens_and_custom_escapes() {
        let mut lexer = Lexer::from(r"a{|}bc\{de\|fg\}hi\n\r\t\0\\");
        lexer.set_escape('\\');
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(Char::raw_vec("a")),
                start: 0,
                end: 1,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                start: 1,
                end: 2,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                start: 2,
                end: 3,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                start: 3,
                end: 4,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![
                    Char::Raw('b'),
                    Char::Raw('c'),
                    Char::Escaped('{', ['\\', '{']),
                    Char::Raw('d'),
                    Char::Raw('e'),
                    Char::Escaped('|', ['\\', '|']),
                    Char::Raw('f'),
                    Char::Raw('g'),
                    Char::Escaped('}', ['\\', '}']),
                    Char::Raw('h'),
                    Char::Raw('i'),
                    Char::Escaped('\n', ['\\', 'n']),
                    Char::Escaped('\r', ['\\', 'r']),
                    Char::Escaped('\t', ['\\', 't']),
                    Char::Escaped('\0', ['\\', '0']),
                    Char::Escaped('\\', ['\\', '\\']),
                ]),
                start: 4,
                end: 28,
            }))
        );
    }
}
