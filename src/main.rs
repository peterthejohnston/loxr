use std::fmt;

const DEBUG: bool = true;
const STACK_MAX: usize = 256;

enum Opcode {
    Return,
    Constant,
    Negate,
    Error,
}

impl From<Opcode> for u8 {
    fn from(opcode: Opcode) -> u8 {
        match opcode {
            Opcode::Return => 0,
            Opcode::Constant => 1,
            Opcode::Negate => 2,
            // This should never be used
            Opcode::Error => std::u8::MAX,
        }
    }
}

impl From<u8> for Opcode {
    fn from(n: u8) -> Opcode {
        match n {
            0 => Opcode::Return,
            1 => Opcode::Constant,
            2 => Opcode::Negate,
            _ => Opcode::Error,
        }
    }
}

#[derive(Copy, Clone)]
enum Value {
    Number(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
        }
    }
}

// Why is it called chunk...
struct Chunk {
    code: Vec<u8>,
    // [run length] [line no] ...
    lines: Vec<usize>,
    constants: Vec<Value>,
}

impl Chunk {
    fn new() -> Chunk {
        Chunk {
            code: vec![],
            lines: vec![],
            constants: vec![],
        }
    }

    fn line_at(&self, offset: usize) -> usize {
        let mut current_line = 0;
        let mut bytes = 0;
        for line_info in self.lines.chunks(2) {
            let (run_length, line_number) = (line_info[0], line_info[1]);
            bytes += run_length;
            if offset > bytes {
                break;
            }
            current_line = line_number;
        }
        current_line
    }

    fn write(&mut self, byte: u8, line_number: usize) {
        self.code.push(byte);
        if !self.lines.is_empty() && self.lines.last().unwrap() == &line_number {
            // We are still on the last line. Increment run length
            let i = self.lines.len() - 2;
            self.lines[i] += 1;
            return;
        }
        // Add an entry for a new line with run length 1
        self.lines.push(1);
        self.lines.push(line_number);
    }

    fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        if offset > 0 && self.line_at(offset) == self.line_at(offset - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", self.line_at(offset));
        }
        match Opcode::from(self.code[offset]) {
            Opcode::Constant => {
                let addr = self.code[offset + 1] as usize;
                println!("{:16} {:4} '{}'", "OP_CONSTANT", addr, self.constants[addr]);
                offset + 2
            },
            Opcode::Negate => {
                println!("OP_NEGATE");
                offset + 1
            },
            Opcode::Return => {
                println!("OP_RETURN");
                offset + 1
            },
            Opcode::Error => {
                println!("INVALID OPCODE");
                std::usize::MAX
            }
        }
    }
}

enum InterpretError {
    CompileError,
    RuntimeError,
}

struct VM {
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: u8,
}

impl VM {
    fn new() -> VM {
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

    fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretError> {
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
                Opcode::Negate => {
                    let Value::Number(n) = self.pop();
                    self.push(Value::Number(-n));
                    self.ip + 1
                },
                _ => return Err(InterpretError::RuntimeError),
            }
        }
    }
}

fn main() {
    let mut chunk = Chunk::new();

    chunk.constants.push(Value::Number(1.2));

    chunk.write(Opcode::Constant as u8, 123);
    chunk.write(0, 123);
    chunk.write(Opcode::Negate as u8, 123);
    chunk.write(Opcode::Return as u8, 123);

    match VM::new().interpret(&chunk) {
        Ok(()) => (),
        Err(InterpretError::CompileError) => println!("Compile error!"),
        Err(InterpretError::RuntimeError) => println!("Runtime error!"),
    }
}
