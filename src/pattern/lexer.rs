use crate::pattern::parse::Parsed;
use crate::pattern::reader::Reader;

const EXPR_START: char = '{';
const EXPR_END: char = '}';
const PIPE: char = '|';

#[derive(Debug, PartialEq)]
pub enum Token {
    Raw(String),
    ExprStart,
    ExprEnd,
    Pipe,
}

pub struct Lexer {
    reader: Reader,
    position: usize,
    character: Option<char>,
    in_expression: bool,
}

impl Iterator for Lexer {
    type Item = Parsed<Token>;

    fn next(&mut self) -> Option<Parsed<Token>> {
        if self.in_expression {
            self.next_in_expresion()
        } else {
            self.next_outside_expression()
        }
    }
}

impl Lexer {
    pub fn new(string: &str) -> Self {
        let mut lexer = Self {
            reader: Reader::new(string),
            position: 0,
            character: None,
            in_expression: false,
        };
        lexer.fetch_character();
        lexer
    }

    fn next_outside_expression(&mut self) -> Option<Parsed<Token>> {
        let mut raw = String::new();

        loop {
            match self.character {
                // '{{' is escaped '{'.
                // '}}' is escaped '}'.
                Some(ch @ EXPR_START) | Some(ch @ EXPR_END) => {
                    if self.reader.peek() == self.character {
                        self.fetch_character();
                        self.fetch_character();
                        raw.push(ch);
                    } else {
                        break;
                    }
                }
                Some(ch) => {
                    raw.push(ch);
                    self.fetch_character();
                }
                None => {
                    break;
                }
            }
        }

        if !raw.is_empty() {
            return self.make_raw(raw);
        }

        match self.character {
            Some(EXPR_START) => {
                self.in_expression = true;
                self.fetch_character();
                self.make_expr_start()
            }
            Some(EXPR_END) => {
                self.fetch_character();
                self.make_expr_end()
            }
            Some(ch) => {
                // Raw token should have been returned previously!
                panic!("Unexpected character {}", ch);
            }
            None => None,
        }
    }

    fn next_in_expresion(&mut self) -> Option<Parsed<Token>> {
        let mut raw = String::new();

        loop {
            match self.character {
                // '|{' is escaped '{'.
                // '||' is escaped '|'.
                // '|}' is escaped '}'.
                Some(PIPE) => {
                    if let Some(ch @ EXPR_START) | Some(ch @ EXPR_END) | Some(ch @ PIPE) =
                        self.reader.peek()
                    {
                        self.fetch_character();
                        self.fetch_character();
                        raw.push(ch);
                    } else {
                        break;
                    }
                }
                Some(EXPR_START) | Some(EXPR_END) | None => break,
                Some(ch) => {
                    self.fetch_character();
                    raw.push(ch);
                }
            }
        }

        if !raw.is_empty() {
            return self.make_raw(raw);
        }

        match self.character {
            Some(EXPR_START) => {
                self.fetch_character();
                self.make_expr_start()
            }
            Some(EXPR_END) => {
                self.in_expression = false;
                self.fetch_character();
                self.make_expr_end()
            }
            Some(PIPE) => {
                self.fetch_character();
                self.make_pipe()
            }
            Some(ch) => {
                // Raw token should have been returned previously!
                panic!("Unexpected character {}", ch);
            }
            None => None,
        }
    }

    fn fetch_character(&mut self) -> Option<char> {
        self.character = self.reader.read();
        self.character
    }

    fn make_raw(&mut self, raw: String) -> Option<Parsed<Token>> {
        self.make_token(Token::Raw(raw))
    }

    fn make_expr_start(&mut self) -> Option<Parsed<Token>> {
        self.make_token(Token::ExprStart)
    }

    fn make_expr_end(&mut self) -> Option<Parsed<Token>> {
        self.make_token(Token::ExprEnd)
    }

    fn make_pipe(&mut self) -> Option<Parsed<Token>> {
        self.make_token(Token::Pipe)
    }

    fn make_token(&mut self, value: Token) -> Option<Parsed<Token>> {
        let start = self.position;
        let end = if self.character.is_some() {
            self.reader.position() - 1 // Next character is already fetched.
        } else {
            self.reader.position() // We are at the end.
        };
        let token = Parsed { value, start, end };
        self.position = end;
        Some(token)
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
    fn raw() {
        let mut lexer = Lexer::new("a");
        lexer.assert_raw("a", 0, 1);
        lexer.assert_none();
    }

    #[test]
    fn long_raw() {
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
    fn escaped_expression_start() {
        let mut lexer = Lexer::new("{{");
        lexer.assert_raw("{", 0, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_expression_end() {
        let mut lexer = Lexer::new("}}");
        lexer.assert_raw("}", 0, 2);
        lexer.assert_none();
    }

    #[test]
    fn pipe_outside_expression() {
        let mut lexer = Lexer::new("|");
        lexer.assert_raw("|", 0, 1);
        lexer.assert_none();
    }

    #[test]
    fn pipe_inside_expression() {
        let mut lexer = Lexer::new("{|");
        lexer.assert_expr_start(0, 1);
        lexer.assert_pipe(1, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_pipe_inside_expression() {
        let mut lexer = Lexer::new("{||");
        lexer.assert_expr_start(0, 1);
        lexer.assert_raw("|", 1, 3);
        lexer.assert_none();
    }

    #[test]
    fn raw_inside_expression() {
        let mut lexer = Lexer::new("{a");
        lexer.assert_expr_start(0, 1);
        lexer.assert_raw("a", 1, 2);
        lexer.assert_none();
    }

    #[test]
    fn long_raw_inside_expression() {
        let mut lexer = Lexer::new("{abc");
        lexer.assert_expr_start(0, 1);
        lexer.assert_raw("abc", 1, 4);
        lexer.assert_none();
    }

    #[test]
    fn expression_start_inside_expression() {
        let mut lexer = Lexer::new("{ {");
        lexer.assert_expr_start(0, 1);
        lexer.assert_raw(" ", 1, 2);
        lexer.assert_expr_start(2, 3);
        lexer.assert_none();
    }

    #[test]
    fn escaped_expression_start_inside_expression() {
        let mut lexer = Lexer::new("{|{");
        lexer.assert_expr_start(0, 1);
        lexer.assert_raw("{", 1, 3);
        lexer.assert_none();
    }

    #[test]
    fn empty_expression() {
        let mut lexer = Lexer::new("{}");
        lexer.assert_expr_start(0, 1);
        lexer.assert_expr_end(1, 2);
        lexer.assert_none();
    }

    #[test]
    fn escaped_expression_end_inside_expression() {
        let mut lexer = Lexer::new("{|}");
        lexer.assert_expr_start(0, 1);
        lexer.assert_raw("}", 1, 3);
        lexer.assert_none();
    }

    #[test]
    fn expression_with_pipe() {
        let mut lexer = Lexer::new("{| }");
        lexer.assert_expr_start(0, 1);
        lexer.assert_pipe(1, 2);
        lexer.assert_raw(" ", 2, 3);
        lexer.assert_expr_end(3, 4);
        lexer.assert_none();
    }

    #[test]
    fn expression_with_raw() {
        let mut lexer = Lexer::new("{a}");
        lexer.assert_expr_start(0, 1);
        lexer.assert_raw("a", 1, 2);
        lexer.assert_expr_end(2, 3);
        lexer.assert_none();
    }

    #[test]
    fn expression_with_long_raw() {
        let mut lexer = Lexer::new("{abc}");
        lexer.assert_expr_start(0, 1);
        lexer.assert_raw("abc", 1, 4);
        lexer.assert_expr_end(4, 5);
        lexer.assert_none();
    }

    #[test]
    fn complex_expression() {
        let mut lexer = Lexer::new("{a|bc|||def|{|}}");
        lexer.assert_expr_start(0, 1);
        lexer.assert_raw("a", 1, 2);
        lexer.assert_pipe(2, 3);
        lexer.assert_raw("bc|", 3, 7);
        lexer.assert_pipe(7, 8);
        lexer.assert_raw("def{}", 8, 15);
        lexer.assert_expr_end(15, 16);
        lexer.assert_none();
    }

    #[test]
    fn complex_escaped_raw() {
        let mut lexer = Lexer::new("{{}}{{{{}}}}a{{b}}c{{{{d}}}}e{{f{{g}}h}}i}}");
        lexer.assert_raw("{}{{}}a{b}c{{d}}e{f{g}h}i}", 0, 43);
        lexer.assert_none();
    }

    #[test]
    fn multiple_expressions() {
        let mut lexer = Lexer::new("{}{a}{bc}");
        lexer.assert_expr_start(0, 1);
        lexer.assert_expr_end(1, 2);
        lexer.assert_expr_start(2, 3);
        lexer.assert_raw("a", 3, 4);
        lexer.assert_expr_end(4, 5);
        lexer.assert_expr_start(5, 6);
        lexer.assert_raw("bc", 6, 8);
        lexer.assert_expr_end(8, 9);
        lexer.assert_none();
    }

    #[test]
    fn multiple_raws_and_expressions() {
        let mut lexer = Lexer::new("a{}bc{de}ghi");
        lexer.assert_raw("a", 0, 1);
        lexer.assert_expr_start(1, 2);
        lexer.assert_expr_end(2, 3);
        lexer.assert_raw("bc", 3, 5);
        lexer.assert_expr_start(5, 6);
        lexer.assert_raw("de", 6, 8);
        lexer.assert_expr_end(8, 9);
        lexer.assert_raw("ghi", 9, 12);
        lexer.assert_none();
    }

    #[test]
    fn multiple_escaped_raws_and_expressions() {
        let mut lexer = Lexer::new("{{}}{{{}}}");
        lexer.assert_raw("{}{", 0, 6);
        lexer.assert_expr_start(6, 7);
        lexer.assert_expr_end(7, 8);
        lexer.assert_raw("}", 8, 10);
        lexer.assert_none();
    }

    #[test]
    fn complex_input() {
        let mut lexer = Lexer::new("name_{{{c}}}.{e|s1-3}");
        lexer.assert_raw("name_{", 0, 7);
        lexer.assert_expr_start(7, 8);
        lexer.assert_raw("c", 8, 9);
        lexer.assert_expr_end(9, 10);
        lexer.assert_raw("}.", 10, 13);
        lexer.assert_expr_start(13, 14);
        lexer.assert_raw("e", 14, 15);
        lexer.assert_pipe(15, 16);
        lexer.assert_raw("s1-3", 16, 20);
        lexer.assert_expr_end(20, 21);
        lexer.assert_none();
    }

    impl Lexer {
        fn assert_none(&mut self) {
            assert_eq!(self.next(), None);
        }

        fn assert_raw(&mut self, raw: &str, start: usize, end: usize) {
            self.assert_token(Token::Raw(raw.to_string()), start, end);
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
            assert_eq!(self.next(), Some(Parsed { value, start, end }));
        }
    }
}
