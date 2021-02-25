use crate::pattern::char::Char;
use crate::pattern::parse::{Error, ErrorKind, Parsed, Result};
use crate::pattern::reader::Reader;
use crate::pattern::symbols::{CR, ESCAPE, EXPR_END, EXPR_START, LF, NUL, PIPE, SEP, TAB};
use std::path::MAIN_SEPARATOR;
use std::result;

#[derive(Debug, PartialEq)]
pub enum Token {
    Raw(Vec<Char>),
    ExprStart,
    ExprEnd,
    Pipe,
}

pub struct Lexer {
    reader: Reader<char>,
    escape: char,
}

impl Lexer {
    pub fn new(string: &str) -> Self {
        Self {
            reader: Reader::from(string),
            escape: ESCAPE,
        }
    }

    pub fn set_escape(&mut self, escape: char) {
        self.escape = escape;
    }

    pub fn read_token(&mut self) -> Result<Option<Parsed<Token>>> {
        let start = self.reader.position();
        let value = match self.reader.peek_char() {
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
        Ok(Some(Parsed {
            value,
            range: start..end,
        }))
    }

    fn read_chars(&mut self) -> Result<Vec<Char>> {
        let mut chars = Vec::new();

        loop {
            match self.reader.peek_char() {
                Some(EXPR_START) | Some(EXPR_END) | Some(PIPE) | None => break,
                Some(value) if value == self.escape => {
                    let start = self.reader.position();
                    self.reader.seek();
                    match self.read_escaped_char() {
                        Ok(char) => chars.push(char),
                        Err(kind) => {
                            let end = self.reader.position();
                            return Err(Error {
                                kind,
                                range: start..end,
                            });
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

    fn read_escaped_char(&mut self) -> result::Result<Char, ErrorKind> {
        if let Some(value) = self.reader.read_char() {
            let escape_sequence = [self.escape, value];
            let escaped_value = match value {
                EXPR_START => EXPR_START,
                EXPR_END => EXPR_END,
                PIPE => PIPE,
                SEP => MAIN_SEPARATOR,
                LF => '\n',
                CR => '\r',
                TAB => '\t',
                NUL => '\0',
                _ if value == self.escape => value,
                _ => return Err(ErrorKind::UnknownEscapeSequence(escape_sequence)),
            };
            Ok(Char::Escaped(escaped_value, escape_sequence))
        } else {
            Err(ErrorKind::UnterminatedEscapeSequence(self.escape))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut lexer = Lexer::new("");
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn raw_char() {
        let mut lexer = Lexer::new("a");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Raw('a')]),
                range: 0..1,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn raw_chars() {
        let mut lexer = Lexer::new("abc");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Raw('a'), Char::Raw('b'), Char::Raw('c')]),
                range: 0..3,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn expr_start() {
        let mut lexer = Lexer::new("{");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                range: 0..1,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn expr_end() {
        let mut lexer = Lexer::new("}");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                range: 0..1,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn pipe() {
        let mut lexer = Lexer::new("|");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                range: 0..1,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_expr_start() {
        let mut lexer = Lexer::new("%{");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('{', ['%', '{'])]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_expr_end() {
        let mut lexer = Lexer::new("%}");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('}', ['%', '}'])]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_pipe() {
        let mut lexer = Lexer::new("%|");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('|', ['%', '|'])]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_separator() {
        let mut lexer = Lexer::new("%/");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped(
                    if cfg!(windows) { '\\' } else { '/' },
                    ['%', '/']
                )]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_lf() {
        let mut lexer = Lexer::new("%n");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('\n', ['%', 'n'])]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_cr() {
        let mut lexer = Lexer::new("%r");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('\r', ['%', 'r'])]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_tab() {
        let mut lexer = Lexer::new("%t");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('\t', ['%', 't'])]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_nul() {
        let mut lexer = Lexer::new("%0");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('\0', ['%', '0'])]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn escaped_escape() {
        let mut lexer = Lexer::new("%%");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('%', ['%', '%'])]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn custom_escape() {
        let mut lexer = Lexer::new(r"\|");
        lexer.set_escape('\\');
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Escaped('|', ['\\', '|'])]),
                range: 0..2,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn unterminated_escape() {
        let mut lexer = Lexer::new("%");
        assert_eq!(
            lexer.read_token(),
            Err(Error {
                kind: ErrorKind::UnterminatedEscapeSequence('%'),
                range: 0..1,
            })
        );
    }

    #[test]
    fn unknown_escape() {
        let mut lexer = Lexer::new("%x");
        assert_eq!(
            lexer.read_token(),
            Err(Error {
                kind: ErrorKind::UnknownEscapeSequence(['%', 'x']),
                range: 0..2,
            })
        );
    }

    #[test]
    fn various_tokens() {
        let mut lexer = Lexer::new("a{|}bc{de|fg}hi");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Raw('a')]),
                range: 0..1,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                range: 1..2,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                range: 2..3,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                range: 3..4,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Raw('b'), Char::Raw('c')]),
                range: 4..6,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                range: 6..7,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Raw('d'), Char::Raw('e')]),
                range: 7..9,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                range: 9..10,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Raw('f'), Char::Raw('g')]),
                range: 10..12,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                range: 12..13,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Raw('h'), Char::Raw('i')]),
                range: 13..15,
            }))
        );
        assert_eq!(lexer.read_token(), Ok(None));
    }

    #[test]
    fn various_tokens_and_escapes() {
        let mut lexer = Lexer::new("a{|}bc%{de%|fg%}hi%n%r%t%0%%");
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Raw('a')]),
                range: 0..1,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                range: 1..2,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                range: 2..3,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                range: 3..4,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![
                    Char::Raw('b'),
                    Char::Raw('c'),
                    Char::Escaped('{', ['%', '{']),
                    Char::Raw('d'),
                    Char::Raw('e'),
                    Char::Escaped('|', ['%', '|']),
                    Char::Raw('f'),
                    Char::Raw('g'),
                    Char::Escaped('}', ['%', '}']),
                    Char::Raw('h'),
                    Char::Raw('i'),
                    Char::Escaped('\n', ['%', 'n']),
                    Char::Escaped('\r', ['%', 'r']),
                    Char::Escaped('\t', ['%', 't']),
                    Char::Escaped('\0', ['%', '0']),
                    Char::Escaped('%', ['%', '%']),
                ]),
                range: 4..28,
            }))
        );
    }

    #[test]
    fn various_tokens_custom_escape() {
        let mut lexer = Lexer::new(r"a{|}bc\{de\|fg\}hi\n\r\t\0\\");
        lexer.set_escape('\\');
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![Char::Raw('a')]),
                range: 0..1,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprStart,
                range: 1..2,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::Pipe,
                range: 2..3,
            }))
        );
        assert_eq!(
            lexer.read_token(),
            Ok(Some(Parsed {
                value: Token::ExprEnd,
                range: 3..4,
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
                range: 4..28,
            }))
        );
    }
}
