use lox::chunk::{Chunk, Opcode};
use lox::value::Value;
use lox::vm::{VM, InterpretError};

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(Value::Number(1.2));
    chunk.write(Opcode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    let constant = chunk.add_constant(Value::Number(3.4));
    chunk.write(Opcode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(Opcode::Add as u8, 123);

    let constant = chunk.add_constant(Value::Number(5.6));
    chunk.write(Opcode::Constant as u8, 123);
    chunk.write(constant as u8, 123);

    chunk.write(Opcode::Div as u8, 123);
    chunk.write(Opcode::Neg as u8, 123);
    chunk.write(Opcode::Return as u8, 123);

    chunk.disassemble("arithmetic calculator");

    match VM::new().interpret(&chunk) {
        Ok(()) => (),
        Err(InterpretError::CompileError) => println!("Compile error!"),
        Err(InterpretError::RuntimeError) => println!("Runtime error!"),
    }
}
