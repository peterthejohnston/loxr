use crate::chunk::{Chunk, Opcode};
use crate::compiler::compile;
use crate::value::{Value, Obj};

pub const DEBUG: bool = false;
// TODO: contrain VM::stack somehow?
const STACK_MAX: usize = 256;

pub enum InterpretError {
    CompileError,
    RuntimeError,
}

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            ip: 0,
            stack: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.ip = 0;
        self.stack = Vec::new();
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn pop(&mut self) -> Result<Value, InterpretError> {
        match self.stack.pop() {
            Some(val) => Ok(val),
            None => Err(InterpretError::RuntimeError), // TODO: StackEmpty
        }
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
        match self.pop()? {
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
        match (self.pop()?, self.pop()?) {
            (Value::Number(rhs), Value::Number(lhs)) => {
                self.push(Value::Number(op(lhs, rhs)));
                Ok(self.ip + 1)
            },
            _ => {
                self.runtime_error(chunk, "Operands must be numbers");
                Err(InterpretError::RuntimeError)
            }
        }
    }

    fn add(&mut self, chunk: &Chunk) -> Result<usize, InterpretError> {
        match (self.pop()?, self.pop()?) {
            (Value::Number(rhs), Value::Number(lhs)) => {
                self.push(Value::Number(lhs + rhs));
                Ok(self.ip + 1)
            },
            // TODO: will have to distinguish between Obj::String and other
            // heap-allocated objects when they exist
            (Value::Obj(box rhs), Value::Obj(box lhs)) => {
                let Obj::String(str_lhs) = lhs;
                let Obj::String(str_rhs) = rhs;
                // TODO: should i clone str_lhs?
                let concat = Box::new(Obj::String(str_lhs + &str_rhs));
                self.push(Value::Obj(concat));
                Ok(self.ip + 1)
            },
            _ => {
                self.runtime_error(chunk, "Operands must be two numbers or two strings");
                Err(InterpretError::RuntimeError)
            }
        }
    }

    fn eq(&mut self, _: &Chunk) -> Result<usize, InterpretError> {
        match (self.pop()?, self.pop()?) {
            (Value::Obj(box rhs), Value::Obj(box lhs)) => {
                // TODO: might need to be different when there are other
                // heap-allocated objects besides strings
                self.push(Value::Bool(rhs == lhs))
            },
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
        match (self.pop()?, self.pop()?) {
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
                println!("\t{:?}", self.stack);
                chunk.disassemble_instruction(self.ip);
            }
            self.ip = match Opcode::from(chunk.code[self.ip]) {
                Opcode::Return => {
                    println!("{}", self.pop()?);
                    return Ok(())
                },
                Opcode::Constant => {
                    let addr = chunk.code[self.ip + 1] as usize;
                    let constant = &chunk.constants[addr];
                    self.push((*constant).clone());
                    self.ip + 2
                },
                Opcode::Nil => { self.push(Value::Nil); self.ip + 1 },
                Opcode::True => { self.push(Value::Bool(true)); self.ip + 1 },
                Opcode::False => { self.push(Value::Bool(false)); self.ip + 1 },
                Opcode::Neg => self.unary_op(chunk, &std::ops::Neg::neg)?,
                Opcode::Not => {
                    let val = self.pop()?;
                    self.push(Value::Bool(self.is_falsey(val)));
                    self.ip + 1
                },
                Opcode::Add => self.add(chunk)?,
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
