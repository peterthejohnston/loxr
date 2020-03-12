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

    fn reset(&mut self) {
        self.ip = 0;
        self.stack = [Value::Number(0.0); STACK_MAX];
        self.stack_top = 0;
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top as usize] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top as usize]
    }

    fn runtime_error(&mut self, chunk: &Chunk, message: &str) {
        eprintln!("{}", message);
        let line = chunk.line_at(self.ip);
        eprintln!("[line {}] in script", line);

        self.reset();
    }

    fn is_falsey(&self, value: Value) -> bool {
        value == Value::Nil || value == Value::Bool(false)
    }

    fn unary_op(&mut self, chunk: &Chunk, op: impl Fn(f64) -> f64
    ) -> Result<usize, InterpretError>
    {
        match self.pop() {
            Value::Number(lhs) => {
                self.push(Value::Number(op(lhs)));
                Ok(self.ip + 1)
            },
            _ => {
                self.runtime_error(chunk, "Operand must be a number");
                Err(InterpretError::RuntimeError)
            }
        }
    }

    fn binary_op(&mut self, chunk: &Chunk, op: impl Fn(f64, f64) -> f64
    ) -> Result<usize, InterpretError>
    {
        // TODO: any way to avoid popping until we know they're Numbers?
        match (self.pop(), self.pop()) {
            (Value::Number(rhs), Value::Number(lhs)) => {
                self.push(Value::Number(op(lhs, rhs)));
                return Ok(self.ip + 1)
            },
            _ => {
                self.runtime_error(chunk, "Operands must be numbers");
                Err(InterpretError::RuntimeError)
            }
        }
    }

    fn eq(&mut self, _: &Chunk) -> Result<usize, InterpretError> {
        match (self.pop(), self.pop()) {
            (Value::Number(rhs), Value::Number(lhs)) => {
                self.push(Value::Bool(rhs == lhs))
            },
            (Value::Bool(rhs), Value::Bool(lhs)) => {
                self.push(Value::Bool(rhs == lhs))
            },
            (Value::Nil, Value::Nil) => {
                self.push(Value::Bool(true))
            },
            _ => {
                self.push(Value::Bool(false))
            },
        };
        Ok(self.ip + 1)
    }

    fn cmp(&mut self, chunk: &Chunk, op: impl Fn(&f64, &f64) -> bool
    ) -> Result<usize, InterpretError>
    {
        match (self.pop(), self.pop()) {
            (Value::Number(rhs), Value::Number(lhs)) => {
                self.push(Value::Bool(op(&lhs, &rhs)));
                return Ok(self.ip + 1)
            },
            _ => {
                self.runtime_error(chunk, "Operands must be numbers");
                Err(InterpretError::RuntimeError)
            }
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretError> {
        self.reset();

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
                Opcode::Nil => { self.push(Value::Nil); self.ip + 1 },
                Opcode::True => { self.push(Value::Bool(true)); self.ip + 1 },
                Opcode::False => { self.push(Value::Bool(false)); self.ip + 1 },
                Opcode::Neg => self.unary_op(chunk, &std::ops::Neg::neg)?,
                Opcode::Not => {
                    let val = self.pop();
                    self.push(Value::Bool(self.is_falsey(val)));
                    self.ip + 1
                },
                Opcode::Add => self.binary_op(chunk, &std::ops::Add::add)?,
                Opcode::Sub => self.binary_op(chunk, &std::ops::Sub::sub)?,
                Opcode::Mul => self.binary_op(chunk, &std::ops::Mul::mul)?,
                Opcode::Div => self.binary_op(chunk, &std::ops::Div::div)?,
                Opcode::Equal => self.eq(chunk)?,
                Opcode::Greater => self.cmp(chunk, std::cmp::PartialOrd::gt)?,
                Opcode::Less => self.cmp(chunk, std::cmp::PartialOrd::lt)?,
                _ => return Err(InterpretError::RuntimeError),
            }
        }
    }
}
