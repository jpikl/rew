use crate::pattern::char::Char;
use crate::pattern::error::{ConfigError, ErrorType};
use crate::pattern::parse::{ParseError, ParseResult, Parsed};
use crate::pattern::reader::Reader;

pub const DEFAULT_ESCAPE: char = '#';

const EXPR_START: char = '{';
const EXPR_END: char = '}';
const PIPE: char = '|';
const LF: char = 'n';
const CR: char = 'r';
const TAB: char = 't';
const NUL: char = '0';

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
    value: Option<char>,
    start: usize,
    end: usize,
}

impl Lexer {
    pub fn new(string: &str) -> Self {
        let mut lexer = Self {
            reader: Reader::from(string),
            escape: DEFAULT_ESCAPE,
            value: None,
            start: 0,
            end: 0,
        };
        lexer.fetch_value();
        lexer
    }

    pub fn set_escape(&mut self, escape: char) -> Result<(), ConfigError> {
        match escape {
            EXPR_START | EXPR_END | PIPE | LF | CR | TAB | NUL => {
                Err(ConfigError::ForbiddenEscapeChar(escape))
            }
            _ => {
                self.escape = escape;
                Ok(())
            }
        }
    }

    fn read_token(&mut self) -> Option<ParseResult<Parsed<Token>>> {
        let start = self.start;
        let value = match self.value? {
            EXPR_START => {
                self.fetch_value();
                Token::ExprStart
            }
            EXPR_END => {
                self.fetch_value();
                Token::ExprEnd
            }
            PIPE => {
                self.fetch_value();
                Token::Pipe
            }
            _ => match self.read_chars() {
                Ok(chars) => Token::Raw(chars),
                Err(error) => return Some(Err(error)),
            },
        };
        let end = self.start;
        Some(Ok(Parsed { value, start, end }))
    }

    fn read_chars(&mut self) -> ParseResult<Vec<Char>> {
        let mut chars = Vec::new();

        while let Some(value) = self.value {
            if value == EXPR_START || value == EXPR_END || value == PIPE {
                break;
            } else if value == self.escape {
                let start = self.start;
                match self.read_escape() {
                    Ok(char) => chars.push(char),
                    Err(typ) => {
                        let end = self.end;
                        return Err(ParseError { typ, start, end });
                    }
                }
            } else {
                chars.push(Char::Raw(value));
            }
            self.fetch_value();
        }

        Ok(chars)
    }

    fn read_escape(&mut self) -> Result<Char, ErrorType> {
        if let Some(value) = self.fetch_value() {
            let escape_sequence = [self.escape, value];
            let escaped_value = match value {
                EXPR_START => EXPR_START,
                EXPR_END => EXPR_END,
                PIPE => PIPE,
                LF => '\n',
                CR => '\r',
                TAB => '\t',
                NUL => '\0',
                _ => {
                    if value == self.escape {
                        value
                    } else {
                        return Err(ErrorType::UnknownEscapeSequence(escape_sequence));
                    }
                }
            };
            Ok(Char::Escaped(escaped_value, escape_sequence))
        } else {
            Err(ErrorType::UnterminatedEscapeSequence(self.escape))
        }
    }

    fn fetch_value(&mut self) -> Option<char> {
        self.start = self.reader.position();
        self.value = self.reader.read_value();
        self.end = self.reader.position();
        self.value
    }
}

impl Iterator for Lexer {
    type Item = ParseResult<Parsed<Token>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        Lexer::new("").assert_none();
    }

    #[test]
    fn raw_char() {
        let mut lexer = Lexer::new("a");
        lexer.assert_raw("a", 0, 1);
        lexer.assert_none();
    }

    #[test]
    fn raw_chars() {
        let mut lexer = Lexer::new("abc");
        lexer.assert_raw("abc", 0, 3);
        lexer.assert_none();
    }

    #[test]
    fn expression_start() {
        let mut lexer = Lexer::new("{");
        lexer.assert_expr_start(0, 1);
        lexer.assert_none();
    }

    #[test]
    fn expression_end() {
        let mut lexer = Lexer::new("}");
        lexer.assert_expr_end(0, 1);
        lexer.assert_none();
    }

    #[test]
    fn pipe() {
        let mut lexer = Lexer::new("|");
        lexer.assert_pipe(0, 1);
        lexer.assert_none();
    }

    #[test]
    fn escaped_expression_start() {
        let mut lexer = Lexer::new("#{");
        lexer.assert_raw_vec(vec![Char::Escaped('{', ['#', '{'])], 0, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_expression_end() {
        let mut lexer = Lexer::new("#}");
        lexer.assert_raw_vec(vec![Char::Escaped('}', ['#', '}'])], 0, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_pipe() {
        let mut lexer = Lexer::new("#|");
        lexer.assert_raw_vec(vec![Char::Escaped('|', ['#', '|'])], 0, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_lf() {
        let mut lexer = Lexer::new("#n");
        lexer.assert_raw_vec(vec![Char::Escaped('\n', ['#', 'n'])], 0, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_cr() {
        let mut lexer = Lexer::new("#r");
        lexer.assert_raw_vec(vec![Char::Escaped('\r', ['#', 'r'])], 0, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_tab() {
        let mut lexer = Lexer::new("#t");
        lexer.assert_raw_vec(vec![Char::Escaped('\t', ['#', 't'])], 0, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_nul() {
        let mut lexer = Lexer::new("#0");
        lexer.assert_raw_vec(vec![Char::Escaped('\0', ['#', '0'])], 0, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_escape() {
        let mut lexer = Lexer::new("##");
        lexer.assert_raw_vec(vec![Char::Escaped('#', ['#', '#'])], 0, 2);
    }

    #[test]
    fn custom_escape() {
        let mut lexer = Lexer::new(r"\|");
        lexer.assert_set_escape('\\');
        lexer.assert_raw_vec(vec![Char::Escaped('|', ['\\', '|'])], 0, 2);
    }

    #[test]
    fn unterminated_escape_error() {
        Lexer::new("#").assert_err(ErrorType::UnterminatedEscapeSequence('#'), 0, 1);
    }

    #[test]
    fn unknown_escape_error() {
        Lexer::new("#x").assert_err(ErrorType::UnknownEscapeSequence(['#', 'x']), 0, 2);
    }

    #[test]
    fn forbidden_custom_escape_error() {
        let mut lexer = Lexer::new("");
        lexer.assert_set_escape_err('{', ConfigError::ForbiddenEscapeChar('{'));
        lexer.assert_set_escape_err('|', ConfigError::ForbiddenEscapeChar('|'));
        lexer.assert_set_escape_err('}', ConfigError::ForbiddenEscapeChar('}'));
        lexer.assert_set_escape_err('n', ConfigError::ForbiddenEscapeChar('n'));
        lexer.assert_set_escape_err('r', ConfigError::ForbiddenEscapeChar('r'));
        lexer.assert_set_escape_err('t', ConfigError::ForbiddenEscapeChar('t'));
        lexer.assert_set_escape_err('0', ConfigError::ForbiddenEscapeChar('0'));
    }

    #[test]
    fn various_tokens() {
        let mut lexer = Lexer::new("a{|}bc{de|fg}hi");
        lexer.assert_raw("a", 0, 1);
        lexer.assert_expr_start(1, 2);
        lexer.assert_pipe(2, 3);
        lexer.assert_expr_end(3, 4);
        lexer.assert_raw("bc", 4, 6);
        lexer.assert_expr_start(6, 7);
        lexer.assert_raw("de", 7, 9);
        lexer.assert_pipe(9, 10);
        lexer.assert_raw("fg", 10, 12);
        lexer.assert_expr_end(12, 13);
        lexer.assert_raw("hi", 13, 15);
    }

    #[test]
    fn various_tokens_and_escapes() {
        let mut lexer = Lexer::new("a{|}bc#{de#|fg#}hi#n#r#t#0##");
        lexer.assert_raw("a", 0, 1);
        lexer.assert_expr_start(1, 2);
        lexer.assert_pipe(2, 3);
        lexer.assert_expr_end(3, 4);
        lexer.assert_raw_vec(
            vec![
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
            ],
            4,
            28,
        );
    }

    #[test]
    fn various_tokens_and_custom_escapes() {
        let mut lexer = Lexer::new(r"a{|}bc\{de\|fg\}hi\n\r\t\0\\");
        lexer.assert_set_escape('\\');
        lexer.assert_raw("a", 0, 1);
        lexer.assert_expr_start(1, 2);
        lexer.assert_pipe(2, 3);
        lexer.assert_expr_end(3, 4);
        lexer.assert_raw_vec(
            vec![
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
            ],
            4,
            28,
        );
    }

    impl Lexer {
        fn assert_none(&mut self) {
            assert_eq!(self.next(), None);
        }

        fn assert_raw(&mut self, raw: &str, start: usize, end: usize) {
            self.assert_token(Token::Raw(Char::raw_vec(raw)), start, end);
        }

        fn assert_raw_vec(&mut self, chars: Vec<Char>, start: usize, end: usize) {
            self.assert_token(Token::Raw(chars), start, end);
        }

        fn assert_expr_start(&mut self, start: usize, end: usize) {
            self.assert_token(Token::ExprStart, start, end);
        }

        fn assert_expr_end(&mut self, start: usize, end: usize) {
            self.assert_token(Token::ExprEnd, start, end);
        }

        fn assert_pipe(&mut self, start: usize, end: usize) {
            self.assert_token(Token::Pipe, start, end);
        }

        fn assert_token(&mut self, value: Token, start: usize, end: usize) {
            assert_eq!(self.next(), Some(Ok(Parsed { value, start, end })));
        }

        fn assert_err(&mut self, typ: ErrorType, start: usize, end: usize) {
            assert_eq!(self.next(), Some(Err(ParseError { typ, start, end })));
        }

        fn assert_set_escape(&mut self, escape: char) {
            assert_eq!(self.set_escape(escape), Ok(()));
        }

        fn assert_set_escape_err(&mut self, escape: char, error: ConfigError) {
            assert_eq!(self.set_escape(escape), Err(error));
        }
    }
}
