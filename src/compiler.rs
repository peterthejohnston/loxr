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

    EOF,
    Error,
}

struct Token<'a> {
    token_type: TokenType,
    lexeme: &'a str,
    line: usize,        // The source line number of the token
}

struct Scanner<'a> {
    source: &'a str,    // The source string to be scanned
    iter: std::iter::Peekable<std::str::Chars<'a>>,
    start: usize,       // The index of the start of the current lexeme
    current: usize,     // The index of the current character
    line: usize,        // The current source line number
}

impl Scanner<'_> {
    fn new(source: &str) -> Scanner {
        Scanner {
            source,
            iter: source.chars().peekable(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            lexeme: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn check(&mut self, expected: char) -> bool {
        if self.current >= self.source.len() {
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

    fn skip_whitespace(&mut self) {
        loop {
            match self.iter.peek() {
                Some(' ') | Some('\r') | Some('\t') => { self.advance(); },
                Some('\n') => { self.line += 1; self.advance(); },
                Some(_) | None => return,
            }
        }
    }

    fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current;

        if self.current >= self.source.len() {
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
            _ => Token {
                token_type: TokenType::Error,
                lexeme: "Unexpected character",
                line: self.line,
            },
        }
    }
}

pub fn compile<'a>(source: &'a str) {
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
