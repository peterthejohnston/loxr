extern crate itertools;
use crate::chunk::Chunk;
use crate::lexer::Lexer;
use crate::token::TokenType;
use crate::vm::InterpretError;

pub fn compile(source: &str) -> Result<Chunk, InterpretError> {
    let mut lexer = Lexer::new(source);

    // TODO: compile
    Ok(Chunk::new())
}
