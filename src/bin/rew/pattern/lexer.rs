use crate::pattern::char::Char;
use crate::pattern::parse::{BaseResult, Error, ErrorKind, Parsed, Result};
use crate::pattern::reader::Reader;
use crate::pattern::symbols::{
    CARRIAGE_RETURN, DIR_SEPARATOR, EXPR_END, EXPR_START, HORIZONTAL_TAB, LINE_FEED, NULL, PIPE,
};
use std::path::MAIN_SEPARATOR;

pub type ParsedToken = Parsed<Token>;

#[derive(Debug, PartialEq)]
pub enum Token {
    Raw(Vec<Char>),
    ExprStart,
    ExprEnd,
    Pipe,
}

#[cfg(test)]
impl Token {
    fn raw(value: &str) -> Self {
        Self::Raw(value.chars().map(Char::Raw).collect())
    }

    fn esc(value: char, sequence: crate::pattern::char::EscapeSequence) -> Self {
        Self::Raw(vec![Char::Escaped(value, sequence)])
    }
}

pub struct Lexer {
    reader: Reader<char>,
    escape: char,
}

impl Lexer {
    pub fn new(input: &str, escape: char) -> Self {
        Self {
            reader: Reader::from(input),
            escape,
        }
    }

    pub fn read_token(&mut self) -> Result<Option<ParsedToken>> {
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
                Some(EXPR_START | EXPR_END | PIPE) | None => break,
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

    fn read_escaped_char(&mut self) -> BaseResult<Char> {
        if let Some(value) = self.reader.read_char() {
            let escape_sequence = [self.escape, value];
            let escaped_value = match value {
                EXPR_START => EXPR_START,
                EXPR_END => EXPR_END,
                PIPE => PIPE,
                DIR_SEPARATOR => MAIN_SEPARATOR,
                LINE_FEED => '\n',
                CARRIAGE_RETURN => '\r',
                HORIZONTAL_TAB => '\t',
                NULL => '\0',
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
    use test_case::test_case;

    use crate::pattern::error::ErrorRange;

    use super::*;

    #[test_case("%",  0..1, ErrorKind::UnterminatedEscapeSequence('%')   ; "unterminated escape sequence")]
    #[test_case("%x", 0..2, ErrorKind::UnknownEscapeSequence(['%', 'x']) ; "unknown escape sequence")]
    fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
        assert_eq!(
            Lexer::new(input, '%').read_token(),
            Err(Error { kind, range })
        );
    }

    #[                  test_case("a",  0..1, Token::raw("a")              ; "raw char")]
    #[                  test_case("ab", 0..2, Token::raw("ab")             ; "raw chars")]
    #[                  test_case("{",  0..1, Token::ExprStart             ; "expr start")]
    #[                  test_case("}",  0..1, Token::ExprEnd               ; "expr end")]
    #[                  test_case("|",  0..1, Token::Pipe                  ; "pipe")]
    #[                  test_case("%{", 0..2, Token::esc('{',  ['%', '{']) ; "escaped expr start")]
    #[                  test_case("%}", 0..2, Token::esc('}',  ['%', '}']) ; "escaped expr end")]
    #[                  test_case("%|", 0..2, Token::esc('|',  ['%', '|']) ; "escaped pipe")]
    #[                  test_case("%n", 0..2, Token::esc('\n', ['%', 'n']) ; "escaped line feed")]
    #[                  test_case("%r", 0..2, Token::esc('\r', ['%', 'r']) ; "escaped carriage return")]
    #[                  test_case("%t", 0..2, Token::esc('\t', ['%', 't']) ; "escaped horizontal tab")]
    #[                  test_case("%0", 0..2, Token::esc('\0', ['%', '0']) ; "escaped null")]
    #[                  test_case("%%", 0..2, Token::esc('%',  ['%', '%']) ; "escaped escape")]
    #[cfg_attr(unix,    test_case("%/", 0..2, Token::esc('/',  ['%', '/']) ; "escaped separator"))]
    #[cfg_attr(windows, test_case("%/", 0..2, Token::esc('\\', ['%', '/']) ; "escaped separator"))]
    fn single_token(input: &str, range: ErrorRange, value: Token) {
        let mut lexer = Lexer::new(input, '%');
        assert_eq!(lexer.read_token(), Ok(Some(Parsed { value, range })));
    }

    #[test_case(0,  0..1,   Token::raw("a")  ; "token 0")]
    #[test_case(1,  1..2,   Token::ExprStart ; "token 1")]
    #[test_case(2,  2..3,   Token::Pipe      ; "token 2")]
    #[test_case(3,  3..4,   Token::ExprEnd   ; "token 3")]
    #[test_case(4,  4..6,   Token::raw("bc") ; "token 4")]
    #[test_case(5,  6..7,   Token::ExprStart ; "token 5")]
    #[test_case(6,  7..9,   Token::raw("de") ; "token 6")]
    #[test_case(7,  9..10,  Token::Pipe      ; "token 7")]
    #[test_case(8,  10..12, Token::raw("fg") ; "token 8")]
    #[test_case(9,  12..13, Token::ExprEnd   ; "token 9")]
    #[test_case(10, 13..15, Token::raw("hi") ; "token 10")]
    fn multiple_tokens(index: usize, range: ErrorRange, value: Token) {
        let mut lexer = Lexer::new("a{|}bc{de|fg}hi", '%');
        for _ in 0..index {
            lexer.read_token().unwrap_or_default();
        }
        assert_eq!(lexer.read_token(), Ok(Some(Parsed { value, range })))
    }

    #[test]
    fn multiple_escape_sequences() {
        assert_eq!(
            Lexer::new("a%{%|%}bc%{de%|fg%}hi%n%r%t%0%%", '%').read_token(),
            Ok(Some(Parsed {
                value: Token::Raw(vec![
                    Char::Raw('a'),
                    Char::Escaped('{', ['%', '{']),
                    Char::Escaped('|', ['%', '|']),
                    Char::Escaped('}', ['%', '}']),
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
                range: 0..31,
            }))
        );
    }

    #[test_case("",   0 ; "empty")]
    #[test_case("a",  1 ; "raw char")]
    #[test_case("ab", 1 ; "raw chars")]
    #[test_case("{",  1 ; "expr start")]
    #[test_case("}",  1 ; "expr end")]
    #[test_case("|",  1 ; "pipe")]
    #[test_case("%{", 1 ; "escaped expr start")]
    #[test_case("%}", 1 ; "escaped expr end")]
    #[test_case("%|", 1 ; "escaped pipe")]
    #[test_case("%/", 1 ; "escaped separator")]
    #[test_case("%n", 1 ; "escaped line feed")]
    #[test_case("%r", 1 ; "escaped carriage return")]
    #[test_case("%t", 1 ; "escaped horizontal tab")]
    #[test_case("%0", 1 ; "escaped null")]
    #[test_case("%%", 1 ; "escaped escape")]
    #[test_case("a{|}bc{de|fg}hi",                 11 ; "multiple tokens")]
    #[test_case("a%{%|%}bc%{de%|fg%}hi%n%r%t%0%%", 1  ; "multiple escape sequences")]
    fn token_count(input: &str, count: usize) {
        let mut lexer = Lexer::new(input, '%');
        for i in 0..count {
            claim::assert_matches!(lexer.read_token(), Ok(Some(_)), "position {}", i);
        }
        assert_eq!(lexer.read_token(), Ok(None), "last position");
    }
}
