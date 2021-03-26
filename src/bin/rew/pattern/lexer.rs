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
    use super::*;
    use crate::pattern::symbols::DEFAULT_ESCAPE;
    use crate::utils::ByteRange;
    use test_case::test_case;

    type EK = ErrorKind;
    type T = Token;
    type C = Char;

    #[test_case("%",  EK::UnterminatedEscapeSequence('%'),   0..1)]
    #[test_case("%x", EK::UnknownEscapeSequence(['%', 'x']), 0..2)]
    fn err(input: &str, kind: ErrorKind, range: ByteRange) {
        assert_eq!(
            Lexer::new(input, DEFAULT_ESCAPE).read_token(),
            Err(Error { kind, range })
        );
    }

    #[cfg_attr(unix,    test_case("%/", Token::Raw(vec![C::Escaped('/',  ['%', '/'])]), 0..2; "escaped separator"))]
    #[cfg_attr(windows, test_case("%/", Token::Raw(vec![C::Escaped('\\', ['%', '/'])]), 0..2; "escaped separator"))]
    #[test_case("a",  T::Raw(vec![C::Raw('a')]),                  0..1; "raw char")]
    #[test_case("ab", T::Raw(vec![C::Raw('a'), C::Raw('b')]),     0..2; "raw chars")]
    #[test_case("{",  T::ExprStart,                               0..1; "expr start")]
    #[test_case("}",  T::ExprEnd,                                 0..1; "expr end")]
    #[test_case("|",  T::Pipe,                                    0..1; "pipe")]
    #[test_case("%{", T::Raw(vec![C::Escaped('{',  ['%', '{'])]), 0..2; "escaped expr start")]
    #[test_case("%}", T::Raw(vec![C::Escaped('}',  ['%', '}'])]), 0..2; "escaped expr end")]
    #[test_case("%|", T::Raw(vec![C::Escaped('|',  ['%', '|'])]), 0..2; "escaped pipe")]
    #[test_case("%n", T::Raw(vec![C::Escaped('\n', ['%', 'n'])]), 0..2; "escaped line feed")]
    #[test_case("%r", T::Raw(vec![C::Escaped('\r', ['%', 'r'])]), 0..2; "escaped carriage return")]
    #[test_case("%t", T::Raw(vec![C::Escaped('\t', ['%', 't'])]), 0..2; "escaped horizontal tab")]
    #[test_case("%0", T::Raw(vec![C::Escaped('\0', ['%', '0'])]), 0..2; "escaped null")]
    #[test_case("%%", T::Raw(vec![C::Escaped('%',  ['%', '%'])]), 0..2; "escaped escape")]
    fn single_token(input: &str, value: Token, range: ByteRange) {
        let mut lexer = Lexer::new(input, DEFAULT_ESCAPE);
        assert_eq!(lexer.read_token(), Ok(Some(Parsed { value, range })));
    }

    #[test_case(0,  T::Raw(vec![C::Raw('a')]),              0..1;   "token 0")]
    #[test_case(1,  T::ExprStart,                           1..2;   "token 1")]
    #[test_case(2,  T::Pipe,                                2..3;   "token 2")]
    #[test_case(3,  T::ExprEnd,                             3..4;   "token 3")]
    #[test_case(4,  T::Raw(vec![C::Raw('b'), C::Raw('c')]), 4..6;   "token 4")]
    #[test_case(5,  T::ExprStart,                           6..7;   "token 5")]
    #[test_case(6,  T::Raw(vec![C::Raw('d'), C::Raw('e')]), 7..9;   "token 6")]
    #[test_case(7,  T::Pipe,                                9..10;  "token 7")]
    #[test_case(8,  T::Raw(vec![C::Raw('f'), C::Raw('g')]), 10..12; "token 8")]
    #[test_case(9,  T::ExprEnd,                             12..13; "token 9")]
    #[test_case(10, T::Raw(vec![C::Raw('h'), C::Raw('i')]), 13..15; "token 10")]
    fn multiple_tokens(index: usize, value: Token, range: ByteRange) {
        let mut lexer = Lexer::new("a{|}bc{de|fg}hi", DEFAULT_ESCAPE);
        for _ in 0..index {
            lexer.read_token().unwrap_or_default();
        }
        assert_eq!(lexer.read_token(), Ok(Some(Parsed { value, range })))
    }

    #[test]
    fn multiple_escape_sequences() {
        assert_eq!(
            Lexer::new("a%{%|%}bc%{de%|fg%}hi%n%r%t%0%%", DEFAULT_ESCAPE).read_token(),
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

    #[test_case("",   0; "empty")]
    #[test_case("a",  1; "raw char")]
    #[test_case("ab", 1; "raw chars")]
    #[test_case("{",  1; "expr start")]
    #[test_case("}",  1; "expr end")]
    #[test_case("|",  1; "pipe")]
    #[test_case("%{", 1; "escaped expr start")]
    #[test_case("%}", 1; "escaped expr end")]
    #[test_case("%|", 1; "escaped pipe")]
    #[test_case("%/", 1; "escaped separator")]
    #[test_case("%n", 1; "escaped line feed")]
    #[test_case("%r", 1; "escaped carriage return")]
    #[test_case("%t", 1; "escaped horizontal tab")]
    #[test_case("%0", 1; "escaped null")]
    #[test_case("%%", 1; "escaped escape")]
    #[test_case("a{|}bc{de|fg}hi",                 11; "multiple tokens")]
    #[test_case("a%{%|%}bc%{de%|fg%}hi%n%r%t%0%%", 1;  "multiple escape sequences")]
    fn token_count(input: &str, count: usize) {
        let mut lexer = Lexer::new(input, DEFAULT_ESCAPE);
        for i in 0..count {
            claim::assert_matches!(lexer.read_token(), Ok(Some(_)), "position {}", i);
        }
        assert_eq!(lexer.read_token(), Ok(None), "last position");
    }
}
