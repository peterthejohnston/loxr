use crate::chunk::{Chunk, Opcode};
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};
use crate::value::Value;
use crate::vm::{DEBUG, InterpretError};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl From<Precedence> for u8 {
    fn from(prec: Precedence) -> u8 {
        match prec {
            Precedence::None       => 0,
            Precedence::Assignment => 1,
            Precedence::Or         => 2,
            Precedence::And        => 3,
            Precedence::Equality   => 4,
            Precedence::Comparison => 5,
            Precedence::Term       => 6,
            Precedence::Factor     => 7,
            Precedence::Unary      => 8,
            Precedence::Call       => 9,
            Precedence::Primary    => 10,
        }
    }
}

impl From<u8> for Precedence {
    fn from(n: u8) -> Precedence {
        match n {
            1 =>  Precedence::Assignment,
            2 =>  Precedence::Or,
            3 =>  Precedence::And,
            4 =>  Precedence::Equality,
            5 =>  Precedence::Comparison,
            6 =>  Precedence::Term,
            7 =>  Precedence::Factor,
            8 =>  Precedence::Unary,
            9 =>  Precedence::Call,
            10 => Precedence::Primary,
            _  => Precedence::None,
        }
    }
}

impl Precedence {
    fn plus_one(&self) -> Precedence {
        (*self as u8 + 1).into()
    }
}

// Rules for a given TokenType
struct ParseRule {
    // The function to compile a prefix expression
    // starting with a token of that type
    prefix: Option<fn(&mut Parser)>,
    // The function to compile an infix expression whose
    // left operand is followed by a token of that type
    infix: Option<fn(&mut Parser)>,
    // The precedence of an infix expression
    // that uses that token as an operator
    precedence: Precedence,
}

fn get_parse_rule(token_type: TokenType) -> ParseRule {
    match token_type {
        TokenType::LeftParen => ParseRule {
            prefix: Some(|parser| parser.grouping()),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::Bang => ParseRule {
            prefix: Some(|parser| parser.unary()),
            infix: None,
            precedence: Precedence::Term,
        },
        TokenType::Minus => ParseRule {
            prefix: Some(|parser| parser.unary()),
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Term,
        },
        TokenType::Plus => ParseRule {
            prefix: None,
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Term,
        },
        TokenType::Slash => ParseRule {
            prefix: None,
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Factor,
        },
        TokenType::Star => ParseRule {
            prefix: None,
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Factor,
        },
        TokenType::EqualEqual => ParseRule {
            prefix: None,
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Equality,
        },
        TokenType::BangEqual => ParseRule {
            prefix: None,
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Equality,
        },
        TokenType::Greater => ParseRule {
            prefix: None,
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Comparison,
        },
        TokenType::GreaterEqual => ParseRule {
            prefix: None,
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Comparison,
        },
        TokenType::Less => ParseRule {
            prefix: None,
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Comparison,
        },
        TokenType::LessEqual => ParseRule {
            prefix: None,
            infix: Some(|parser| parser.binary()),
            precedence: Precedence::Comparison,
        },
        TokenType::Number => ParseRule {
            prefix: Some(|parser| parser.number()),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::False => ParseRule {
            prefix: Some(|parser| parser.literal()),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::True => ParseRule {
            prefix: Some(|parser| parser.literal()),
            infix: None,
            precedence: Precedence::None,
        },
        TokenType::Nil => ParseRule {
            prefix: Some(|parser| parser.literal()),
            infix: None,
            precedence: Precedence::None,
        },
        _ => ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        },
    }
}

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

    fn literal(&mut self) {
        match self.previous.token_type {
            TokenType::False => self.emit_constant(Value::Bool(false)),
            TokenType::True => self.emit_constant(Value::Bool(true)),
            TokenType::Nil => self.emit_constant(Value::Nil),
            _ => (), // Should be unreachable
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn unary(&mut self) {
        let op_type = self.previous.token_type;
        self.parse_precedence(Precedence::Unary);

        match op_type {
            TokenType::Minus => self.emit_byte(Opcode::Neg.into()),
            TokenType::Bang => self.emit_byte(Opcode::Not.into()),
            _ => (), // TODO: Should be unreachable
        };
    }

    fn binary(&mut self) {
        let op_type = self.previous.token_type;

        let rule = get_parse_rule(op_type);
        self.parse_precedence(rule.precedence.plus_one());
        // TODO: error, no rule for token '${op_type}' as a bin operator
        // if it's a default ParseRule?

        match op_type {
            TokenType::Plus => self.emit_byte(Opcode::Add.into()),
            TokenType::Minus => self.emit_byte(Opcode::Sub.into()),
            TokenType::Star => self.emit_byte(Opcode::Mul.into()),
            TokenType::Slash => self.emit_byte(Opcode::Div.into()),
            TokenType::EqualEqual => self.emit_byte(Opcode::Equal.into()),
            TokenType::BangEqual => { self.emit_byte(Opcode::Equal.into()); self.emit_byte(Opcode::Not.into()); },
            TokenType::Greater => self.emit_byte(Opcode::Greater.into()),
            TokenType::GreaterEqual => { self.emit_byte(Opcode::Less.into()); self.emit_byte(Opcode::Not.into()); },
            TokenType::Less => self.emit_byte(Opcode::Less.into()),
            TokenType::LessEqual => { self.emit_byte(Opcode::Greater.into()); self.emit_byte(Opcode::Not.into()); },
            _ => (), // TODO: Should never happen
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence(&mut self, prec: Precedence) {
        self.advance();
        let prefix_rule = get_parse_rule(self.previous.token_type);
        if let Some(prefix_fn) = prefix_rule.prefix {
            prefix_fn(self);
        } else {
            self.error("Expect expression");
            return;
        }

        while prec <= get_parse_rule(self.current.token_type).precedence {
            self.advance();
            let infix_rule = get_parse_rule(self.previous.token_type);
            if let Some(infix_fn) = infix_rule.infix {
                infix_fn(self);
            } else {
                // TODO: is this the error i want
                self.error("Expect expression");
                return;
            }
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        self.chunk.write(byte, self.previous.line);
    }

    fn emit_constant(&mut self, value: Value) {
        self.emit_byte(Opcode::Constant.into());
        let i = self.chunk.add_constant(value);
        self.emit_byte(i as u8);
    }
}

pub fn compile(source: &str) -> Result<Chunk, InterpretError> {
    let mut parser = Parser::new(source);

    parser.advance();
    parser.expression();
    parser.consume(TokenType::EOF, "Expect end of expression");
    parser.emit_byte(Opcode::Return.into());

    if DEBUG && !parser.had_error {
        parser.chunk.disassemble("code");
    }

    if parser.had_error {
        Err(InterpretError::CompileError)
    } else {
        Ok(parser.chunk)
    }
}
