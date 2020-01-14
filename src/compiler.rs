#[derive(Debug)]
enum TokenType {
    EOF,
}

struct Token<'a> {
    token_type: TokenType,
    lexeme: &'a str,
    line: usize,    // The source line number of the token
}

struct Scanner<'a> {
    source: &'a str,   // The source string to be scanned
    start: usize,   // The index of the start of the current lexeme
    current: usize, // The index of the current character
    line: usize,    // The current source line number
}

impl Scanner<'_> {
    fn new(source: &str) -> Scanner {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn scan_token(&self) -> Token {
        Token {
            token_type: TokenType::EOF,
            lexeme: &self.source[..1],
            line: 1,
        }
    }
}

pub fn compile<'a>(source: &'a str) {
    let scanner = Scanner::new(source);
    let mut line = 0;
    loop {
        let token = scanner.scan_token();
        if token.line != line {
            print!("{:04} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }

        println!("{:?} {}", token.token_type, token.lexeme);

        match token.token_type {
            TokenType::EOF => break,
        }
    }
}
