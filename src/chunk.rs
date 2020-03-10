use crate::value::Value;

pub enum Opcode {
    Return,
    Constant,
    Nil,
    True,
    False,
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Error,
}

impl From<Opcode> for u8 {
    fn from(opcode: Opcode) -> u8 {
        match opcode {
            Opcode::Return   => 0,
            Opcode::Constant => 1,
            Opcode::Nil      => 2,
            Opcode::True     => 3,
            Opcode::False    => 4,
            Opcode::Neg      => 5,
            Opcode::Add      => 6,
            Opcode::Sub      => 7,
            Opcode::Mul      => 8,
            Opcode::Div      => 9,
            // This should never be used
            Opcode::Error    => std::u8::MAX,
        }
    }
}

impl From<u8> for Opcode {
    fn from(n: u8) -> Opcode {
        match n {
            0 => Opcode::Return,
            1 => Opcode::Constant,
            2 => Opcode::Nil,
            3 => Opcode::True,
            4 => Opcode::False,
            5 => Opcode::Neg,
            6 => Opcode::Add,
            7 => Opcode::Sub,
            8 => Opcode::Mul,
            9 => Opcode::Div,
            _ => Opcode::Error,
        }
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
    // [run length] [line no] ...
    lines: Vec<usize>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: vec![],
            lines: vec![],
            constants: vec![],
        }
    }

    pub fn line_at(&self, offset: usize) -> usize {
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

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn write(&mut self, byte: u8, line_number: usize) {
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

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
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
            Opcode::Nil => { println!("OP_NIL"); offset + 1 },
            Opcode::True => { println!("OP_TRUE"); offset + 1 },
            Opcode::False => { println!("OP_FALSE"); offset + 1 },
            Opcode::Neg => { println!("OP_NEG"); offset + 1 },
            Opcode::Add => { println!("OP_ADD"); offset + 1 },
            Opcode::Sub => { println!("OP_SUB"); offset + 1 },
            Opcode::Mul => { println!("OP_MUL"); offset + 1 },
            Opcode::Div => { println!("OP_DIV"); offset + 1 },
            Opcode::Return => { println!("OP_RETURN"); offset + 1 },
            Opcode::Error => {
                println!("INVALID OPCODE");
                std::usize::MAX
            }
        }
    }
}
