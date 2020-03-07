#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
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

    // Keywords
    And, Class, Else, False,
    For, Fun, If, Nil, Or,
    Print, Return, Super, This,
    True, Var, While,

    EOF,
    Error,
}

#[derive(Clone, Copy)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub line: usize,        // The source line number of the token
}
