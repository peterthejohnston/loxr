extern crate itertools;
use crate::chunk::{Chunk, Opcode};
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};
use crate::value::Value;
use crate::vm::InterpretError;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    chunk: Chunk,
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panic_mode: bool, // Used for recoverable parsing
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Parser {
        Parser {
            lexer: Lexer::new(source),
            chunk: Chunk::new(),
            // TODO: Find a better pattern for this
            // (what should current and previous be when they are not meaningful)
            current: Token {
                token_type: TokenType::Error,
                lexeme: "",
                line: 0,
            },
            previous: Token {
                token_type: TokenType::Error,
                lexeme: "",
                line: 0,
            },
            had_error: false,
            panic_mode: false,
        }
    }

    // ===================================
    // Frontend (eating tokens)
    // ===================================
    fn advance(&mut self) {
        self.previous = self.current;

        // Read and report error tokens, stop when we hit a non-error
        loop {
            self.current = self.lexer.lex_token();
            match self.current.token_type {
                TokenType::Error => self.error_at_current(self.current.lexeme),
                _ => break,
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance();
        } else {
            self.error_at_current(message);
        }
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous, message)
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current, message)
    }

    fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode { return; }
        self.panic_mode = true;

        eprint!("[line {}] Error", self.current.line);
        let loc = match token.token_type {
            TokenType::EOF => " at end".to_owned(),
            TokenType::Error => "".to_owned(),
            _ => format!(" at '{}'", token.lexeme),
        };
        eprintln!("{}: {}", loc, message);

        self.had_error = true;
    }

    // ===================================
    // Backend (bytecode gen)
    // ===================================
    fn number(&mut self) {
        // TODO: handle parse error
        let n = self.previous.lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value::Number(n));
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn expression(&self) {
        // TODO:
        // ...
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte, self.previous.line);
    }

    fn emit_constant(&mut self, value: Value) {
        self.emit_byte(Opcode::Constant as u8);
        let i = self.chunk.add_constant(value);
        self.emit_byte(i as u8);
    }
}

// Where it all comes together
pub fn compile(source: &str) -> Result<Chunk, InterpretError> {
    let mut parser = Parser::new(source);

    parser.advance();
    parser.expression();
    parser.consume(TokenType::EOF, "Expect end of expression");
    parser.emit_byte(Opcode::Return as u8);

    if parser.had_error {
        Err(InterpretError::CompileError)
    } else {
        Ok(parser.chunk)
    }
}
