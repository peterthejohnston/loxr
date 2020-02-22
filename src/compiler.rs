extern crate itertools;
use itertools::{MultiPeek, multipeek};
use std::str::Chars;

#[derive(Debug)]
enum TokenType {
    // One-character tokens
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Semicolon, Comma, Dot,
    Minus, Plus, Slash, Star,
    Bang, Equal, Less, Greater,

    // Two-character tokens
    BangEqual, EqualEqual, LessEqual, GreaterEqual,

    // Literals
    String, Number, Identifier,

    EOF,
    Error,
}

struct Token<'a> {
    token_type: TokenType,
    lexeme: &'a str,
    line: usize,        // The source line number of the token
}

struct Scanner<'a> {
    source: &'a str,     // The source string to be scanned
    iter: MultiPeek<Chars<'a>>,
    start: usize,       // The index of the start of the current lexeme
    current: usize,     // The index of the current character
    line: usize,        // The current source line number
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Scanner {
            source,
            iter: multipeek(source.chars()),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn check(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else if self.iter.peek().unwrap() != &expected {
            false
        } else {
            self.current += 1;
            self.iter.next().unwrap();
            true
        }
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.iter.next().unwrap()
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            lexeme: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn error_token(&self, msg: &'a str) -> Token<'a> {
        Token {
            token_type: TokenType::Error,
            lexeme: msg,
            line: self.line,
        }
    }

    fn string_token(&mut self) -> Token {
        while !self.is_at_end() {
            match self.iter.peek() {
                Some('"') => break,
                // Support multi-line strings
                Some('\n') => {
                    self.line += 1;
                    self.advance()
                },
                _ => self.advance()
            };
        }
        self.iter.reset_peek();

        if self.is_at_end() {
            self.error_token("Unterminated string")
        } else {
            self.advance();
            self.make_token(TokenType::String)
        }
    }

    fn consume_digits(&mut self) {
        while let Some('0'..='9') = self.iter.peek() {
            self.advance();
        }
        self.iter.reset_peek();
    }

    fn number_token(&mut self) -> Token {
        self.consume_digits();

        // Look for a fractional part
        match self.iter.peek() {
            Some('.') => {
                // Second char of lookahead to see if we should consume '.'
                self.consume_digits();
            },
            _ => (),
        }
        self.iter.reset_peek();

        self.make_token(TokenType::Number)
    }

    fn identifier_token(&mut self) -> Token {
        while let Some(c) = self.iter.peek() {
            if c.is_alphanumeric() {
                self.advance();
            } else {
                break;
            }
        }
        self.iter.reset_peek();

        self.make_token(TokenType::Identifier)
    }

    fn skip_comment(&mut self) {
        while !self.is_at_end() {
            match self.iter.peek() {
                // A comment goes until the end of the line.
                Some('\n') => break,
                _ => self.advance(),
            };
        }
        self.iter.reset_peek();
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.iter.peek() {
                Some(' ') | Some('\r') | Some('\t') => { self.advance(); },
                Some('\n') => {
                    self.line += 1;
                    self.advance();
                },
                Some('/') => {
                    // Second char of lookahead
                    match self.iter.peek() {
                        Some('/') => self.skip_comment(),
                        _ => return,
                    };
                },
                _ => return,
            };
        }
    }

    fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        match self.advance() {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::Semicolon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '/' => self.make_token(TokenType::Slash),
            '*' => self.make_token(TokenType::Star),
            '!' => {
                let token_type =
                    if self.check('=') { TokenType::BangEqual }
                    else { TokenType::Bang };
                self.make_token(token_type)
            },
            '=' => {
                let token_type =
                    if self.check('=') { TokenType::EqualEqual }
                    else { TokenType::Equal };
                self.make_token(token_type)
            },
            '<' => {
                let token_type =
                    if self.check('=') { TokenType::LessEqual }
                    else { TokenType::Less };
                self.make_token(token_type)
            },
            '>' => {
                let token_type =
                    if self.check('=') { TokenType::GreaterEqual }
                    else { TokenType::Greater };
                self.make_token(token_type)
            },
            '"' => self.string_token(),
            '0'..='9' => self.number_token(),
            c if c.is_alphabetic() => self.identifier_token(),
            _ => self.error_token("Unexpected character"),
        }
    }
}

pub fn compile(source: &str) {
    let mut scanner = Scanner::new(source);
    let mut line = 0;
    loop {
        let token = scanner.scan_token();
        if token.line != line {
            print!("{:04} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }

        println!("{:?} '{}'", token.token_type, token.lexeme);

        match token.token_type {
            TokenType::EOF => break,
            _ => (),
        }
    }
}
