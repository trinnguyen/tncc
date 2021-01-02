use std::str::Chars;

use crate::common::{TokType, Token};

/// scan the input source code into array of tokens
pub fn scan(src: &str) -> Vec<Token> {
    let input = ScanInput::from(src.chars());
    input
        .into_iter()
        .collect()
}

#[derive(Debug)]
struct ScanInput<'a> {
    chars: Chars<'a>,
    lookahead: Option<char>,
    line: u32,
    col: u32,
}

/// token iterator for input
impl<'a> Iterator for ScanInput<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.scan_token()
    }
}

/// next and peek operator with single char lookahead
impl<'a> ScanInput<'a> {
    /// scan next token
    fn scan_token(&mut self) -> Option<Token> {
        // skip whitespace
        self.skip_whitespace();

        // cache column
        let col = self.col;

        // start with letter -> ID or keyword
        // underscore is allowed
        match self.next() {
            None => None,
            Some(c) => {
                let typ = match c {
                    '(' => TokType::ParentOpen,
                    ')' => TokType::ParentClose,
                    '{' => TokType::BracketOpen,
                    '}' => TokType::BracketClose,
                    ';' => TokType::Semicolon,
                    '-' => TokType::Minus,
                    '+' => TokType::Plus,
                    '=' => TokType::Assign,
                    t if t.is_ascii_alphabetic() => self.scan_keyword_or_id(t),
                    t if t.is_ascii_digit() => self.scan_num(t),
                    t => panic!("unexpected char: {}", t),
                };
                Some(self.new_token(typ, col))
            }
        }
    }

    /// skip whitespace, tabs and new line
    fn skip_whitespace(&mut self) {
        loop {
            match self.next() {
                Some(c) if c.is_ascii_whitespace() => (),
                Some(c) => {
                    self.put_back(c);
                    break;
                }
                _ => break,
            }
        }
    }

    /// scan id or keyword, id is a sequences of letter or digit, _
    /// start with a letter
    fn scan_keyword_or_id(&mut self, c: char) -> TokType {
        let mut str = String::new();
        str.push(c);
        loop {
            match self.next() {
                Some(c) if c.is_ascii_alphanumeric() || c == '_' => str.push(c),
                Some(c) => {
                    self.put_back(c);
                    break;
                }
                None => break,
            }
        }

        // keywords have higher priority
        match str.as_str() {
            "int" => TokType::KeywordInt,
            "void" => TokType::KeywordVoid,
            "return" => TokType::KeywordReturn,
            _ => TokType::ID(str),
        }
    }

    /// scan positive number: int or double
    fn scan_num(&mut self, c: char) -> TokType {
        let (num1, _) = self.scan_pos_num(self.char_to_u64(c));
        match self.next() {
            Some('.') => {
                let (num2, ct) = self.scan_pos_num(0);
                let real: f64 = num1 as f64 + (num2 as f64).powi(-(ct as i32));
                TokType::NumReal(real)
            }
            Some(c) => {
                self.put_back(c);
                TokType::NumInt(num1)
            }
            _ => TokType::NumInt(num1),
        }
    }

    /// scan positive natural number
    fn scan_pos_num(&mut self, prefix: u64) -> (u64, u32) {
        let mut num = prefix;
        let mut count = 0;
        loop {
            match self.next() {
                Some(c) if c.is_ascii_digit() => {
                    num = num * 10 + self.char_to_u64(c);
                    count = count + 1;
                }
                Some(c) => {
                    self.put_back(c);
                    break;
                }
                None => break,
            }
        }
        (num, count)
    }

    fn char_to_u64(&self, ch: char) -> u64 {
        ch.to_digit(10).unwrap() as u64
    }

    fn new_token(&mut self, tok_type: TokType, col: u32) -> Token {
        Token {
            tok: tok_type,
            loc: (self.line, col),
        }
    }

    /// next character
    fn next(&mut self) -> Option<char> {
        let opt = match self.lookahead {
            Some(c) => {
                self.lookahead = None;
                Some(c)
            }
            _ => self.chars.next(),
        };

        // advance column and line
        if let Some(c) = opt {
            if c == '\n' || c == '\r' {
                self.line = self.line + 1;
                self.col = 1;
            } else {
                self.col = self.col + 1;
            }
        };
        opt
    }

    fn put_back(&mut self, ch: char) {
        self.lookahead = Some(ch);
        self.col = self.col - 1;
    }
}

impl<'a> From<Chars<'a>> for ScanInput<'a> {
    fn from(chs: Chars<'a>) -> Self {
        ScanInput {
            chars: chs,
            lookahead: None,
            line: 1,
            col: 1,
        }
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use crate::common::TokType;

    use super::scan;

    #[test_case("int return void main")]
    #[test_case("1 1.1 0 0.2")]
    #[test_case("a var1")]
    #[test_case("int () ( ) {} { } ; =")]
    fn valid_tokens(src: &str) {
        assert_eq!(!scan(src).is_empty(), true);
    }

    #[test_case("int main() { return 1; }")]
    #[test_case("int main() { int a = 100; return 1; }")]
    fn valid_program(src: &str) {
        assert_eq!(!scan(src).is_empty(), true);
    }

    #[test_case("void", TokType::KeywordVoid)]
    #[test_case("voida", TokType::ID(String::from("voida")))]
    fn single_token(src: &str, tok: TokType) {
        let toks = scan(src);
        assert_eq!(toks.first().unwrap().tok, tok);
    }
}
