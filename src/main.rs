use std::fmt;

enum Byte {
    Opcode(Opcode),
    Param(usize),
}

enum Opcode {
    Constant,
    Return,
}

enum Value {
    Number(f32),
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
    code: Vec<Byte>,
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

    fn write(&mut self, byte: Byte, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
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
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }
        match self.code[offset] {
            Byte::Opcode(Opcode::Constant) => {
                if let Byte::Param(addr) = self.code[offset + 1] {
                    println!("{:16} {:4} '{}'", "OP_CONSTANT", addr, self.constants[addr]);
                } else {
                    println!("ERROR");
                }
                offset + 2
            },
            Byte::Opcode(Opcode::Return) => {
                println!("OP_RETURN");
                offset + 1
            },
            _ => {
                println!("ERROR");
                std::usize::MAX
            }
        }
    }
}

fn main() {
    let mut chunk = Chunk::new();

    chunk.constants.push(Value::Number(1.2));

    chunk.write(Byte::Opcode(Opcode::Constant), 123);
    chunk.write(Byte::Param(0), 123);
    chunk.write(Byte::Opcode(Opcode::Return), 123);

    chunk.disassemble("test chunk");
}
