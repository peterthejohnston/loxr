use crate::chunk::{Chunk, Opcode};
use crate::compiler::compile;
use crate::value::Value;

pub const DEBUG: bool = true;
const STACK_MAX: usize = 256;

pub enum InterpretError {
    CompileError,
    RuntimeError,
}

pub struct VM {
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: u8,
}

impl VM {
    pub fn new() -> VM {
        VM {
            ip: 0,
            stack: [Value::Number(0.0); STACK_MAX],
            stack_top: 0,
        }
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top as usize] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top as usize]
    }

    fn unary_op(&mut self, op: impl Fn(f64) -> f64) -> usize {
        let Value::Number(lhs) = self.pop();
        self.push(Value::Number(op(lhs)));
        self.ip + 1
    }

    fn binary_op(&mut self, op: impl Fn(f64, f64) -> f64) -> usize {
        let Value::Number(rhs) = self.pop();
        let Value::Number(lhs) = self.pop();
        self.push(Value::Number(op(lhs, rhs)));
        self.ip + 1
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretError> {
        let chunk = compile(source)?;

        self.interpret_chunk(&chunk)
    }

    pub fn interpret_chunk(&mut self, chunk: &Chunk) -> Result<(), InterpretError> {
        loop {
            if DEBUG {
                // Print stack
                print!("\t");
                let mut i = 0;
                while i < self.stack_top {
                    print!("[ {} ]", self.stack[i as usize]);
                    i += 1;
                }
                println!("");
                chunk.disassemble_instruction(self.ip);
            }
            self.ip = match Opcode::from(chunk.code[self.ip]) {
                Opcode::Return => {
                    println!("{}", self.pop());
                    return Ok(());
                },
                Opcode::Constant => {
                    let addr = chunk.code[self.ip + 1] as usize;
                    let constant = &chunk.constants[addr];
                    self.push(*constant);
                    self.ip + 2
                },
                Opcode::Neg => self.unary_op(&std::ops::Neg::neg),
                Opcode::Add => self.binary_op(&std::ops::Add::add),
                Opcode::Sub => self.binary_op(&std::ops::Sub::sub),
                Opcode::Mul => self.binary_op(&std::ops::Mul::mul),
                Opcode::Div => self.binary_op(&std::ops::Div::div),
                _ => return Err(InterpretError::RuntimeError),
            }
        }
    }
}
