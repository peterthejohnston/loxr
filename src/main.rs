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

    fn write(&mut self, byte: Byte, line_number: usize) {
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
