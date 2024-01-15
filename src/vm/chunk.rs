use super::value::{Value, ConstantArray};

pub const OP_RETURN: u8 = 0x00;
pub const OP_CONSTANT: u8 = 0x01;
pub const OP_NEGATIVE: u8 = 0x02;
pub const OP_ADD: u8 = 0x03;
pub const OP_SUBTRACT: u8 = 0x04;
pub const OP_MULTIPLY: u8 = 0x05;
pub const OP_DIVIDE: u8 = 0x06;
pub const OP_NULL: u8 = 0x07;
pub const OP_TRUE: u8 = 0x08;
pub const OP_FALSE: u8 = 0x09;
pub const OP_NOT: u8 = 0x0A;
pub const OP_GREATER: u8 = 0x0B;
pub const OP_LESS: u8 = 0x0C;
pub const OP_EQUAL: u8 = 0x0D;
pub const OP_PRINT: u8 = 0x0E;
pub const OP_POP: u8 = 0x0F;
pub const OP_DEFINE_GLOBAL: u8 = 0x10;
pub const OP_GET_GLOBAL: u8 = 0x11;
pub const OP_SET_GLOBAL: u8 = 0x12;
pub const OP_GET_LOCAL: u8 = 0x13;
pub const OP_SET_LOCAL: u8 = 0x14;
pub const OP_JUMP_IF_FALSE: u8 = 0x15;
pub const OP_JUMP: u8 = 0x16;
pub const OP_LOOP: u8 = 0x17;
pub const OP_CALL: u8 = 0x18;
pub const OP_ARRAY: u8 = 0x19;
pub const OP_INDEX_CALL: u8 = 0x1A;
pub const OP_REM: u8 = 0x1B;
pub const OP_POW: u8 = 0x1C;
pub const OP_CLOSURE: u8 = 0x1D;
pub const OP_CLOSE_UPVALUE: u8 = 0x1E;
pub const OP_GET_UPVALUE: u8 = 0x1F;
pub const OP_SET_UPVALUE: u8 = 0x20;
pub const OP_CLASS: u8 = 0x21;
pub const OP_GET_PROP: u8 = 0x22;
pub const OP_SET_PROP: u8 = 0x23;
pub const OP_METHOD: u8 = 0x24;
pub const OP_INVOKE: u8 = 0x25;
pub const OP_INHERIT: u8 = 0x26;
pub const OP_SUPER_INVOKE: u8 = 0x27;
pub const OP_GET_SUPER: u8 = 0x28;
pub const OP_INDEX_SET: u8 = 0x29;
pub const OP_CONSTANT0: u8 = 0x2A;
pub const OP_JUMP_IF_RANGE_END: u8 = 0x2B;
pub const OP_COUNTUP: u8 = 0x2C;
pub const OP_RANGE: u8 = 0x2D;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Chunk {
    code: Vec<u8>,
    constants: ConstantArray,
    lines: Vec<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: ConstantArray::new(),
            lines: Vec::new(),
        }
    }

    pub fn print_instruction(&self) {
        println!(
            "{}",
            self.code
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    pub fn write(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn read_u8(&self, index: usize) -> Option<u8> {
        let u = match self.code.get(index) {
            Some(ip) => *ip,
            None => return None,
        };
        Some(u)
    }

    pub fn read_u16(&self, index: usize) -> Option<u16> {
        let u = match self.read_u8(index) {
            Some(ip) => (ip as u16) << 8,
            None => return None,
        };
        let l = match self.read_u8(index + 1) {
            Some(ip) => ip as u16,
            None => return None,
        };
        Some(u | l)
    }

    pub fn read_u32(&self, index: usize) -> Option<u32> {
        let b1 = match self.read_u8(index) {
            Some(ip) => (ip as u32) << 24,
            None => return None,
        };
        let b2 = match self.read_u8(index + 1) {
            Some(ip) => (ip as u32) << 16,
            None => return None,
        };
        let b3 = match self.read_u8(index + 2) {
            Some(ip) => (ip as u32) << 8,
            None => return None,
        };
        let b4 = match self.read_u8(index + 3) {
            Some(ip) => ip as u32,
            None => return None,
        };
        Some(b1 | b2 | b3 | b4)
    }

    pub fn get_instruction(&self, index: usize) -> Option<&u8> {
        self.code.get(index)
    }

    pub fn get_instruction_mut(&mut self, index: usize) -> Option<&mut u8> {
        self.code.get_mut(index)
    }

    pub fn get_instruction_len(&self) -> usize {
        self.code.len()
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        match (self.constants.len() - 1).try_into() {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }

    pub fn get_constant(&self, index: usize) -> Value {
        self.constants.get(index)
    }

    pub fn get_constant_len(&self) -> usize {
        self.constants.len()
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04X}   | ", offset);
        let instruction = match self.code.get(offset) {
            Some(inst) => inst,
            None => {
                println!("out of code index.({:04X})", offset);
                return 0;
            },
        };

        match *instruction {
            OP_RETURN => self.simple_instruction("OP_RETURN", offset),
            OP_CONSTANT => self.constant_instruction("OP_CONSTANT", offset),
            OP_NEGATIVE => self.simple_instruction("OP_NEGATIVE", offset),
            OP_ADD => self.simple_instruction("OP_ADD", offset),
            OP_SUBTRACT => self.simple_instruction("OP_SUBTRACT", offset),
            OP_MULTIPLY => self.simple_instruction("OP_MULTIPLY", offset),
            OP_DIVIDE => self.simple_instruction("OP_DIVIDE", offset),
            OP_TRUE => self.simple_instruction("OP_TRUE", offset),
            OP_FALSE => self.simple_instruction("OP_FALSE", offset),
            OP_NULL => self.simple_instruction("OP_NULL", offset),
            OP_NOT => self.simple_instruction("OP_NOT", offset),
            OP_GREATER => self.simple_instruction("OP_GREATER", offset),
            OP_LESS => self.simple_instruction("OP_LESS", offset),
            OP_EQUAL => self.simple_instruction("OP_EQUAL", offset),
            OP_PRINT => self.simple_instruction("OP_PRINT", offset),
            OP_POP => self.simple_instruction("OP_POP", offset),
            OP_DEFINE_GLOBAL => self.simple_instruction("OP_DEFINE_GLOBAL", offset),
            OP_GET_GLOBAL => self.simple_instruction("OP_GET_GLOBAL", offset),
            OP_SET_GLOBAL => self.simple_instruction("OP_SET_GLOBAL", offset),
            OP_GET_LOCAL => self.simple_instruction("OP_GET_LOCAL", offset),
            OP_SET_LOCAL => self.simple_instruction("OP_SET_LOCAL", offset),
            OP_JUMP_IF_FALSE => self.jump_instruction("OP_JUMP_IF_FALSE", offset),
            OP_JUMP => self.jump_instruction("OP_JUMP", offset),
            OP_LOOP => self.loop_instruction("OP_LOOP", offset),
            OP_CALL => self.simple_instruction("OP_CALL", offset),
            OP_ARRAY => self.simple_instruction("OP_ARRAY", offset),
            OP_INDEX_CALL => self.simple_instruction("OP_INDEX_CALL", offset),
            OP_REM => self.simple_instruction("OP_REM", offset),
            OP_POW => self.simple_instruction("OP_POW", offset),
            OP_CLOSURE => self.simple_instruction("OP_CLOSURE", offset),
            OP_CLOSE_UPVALUE => self.simple_instruction("OP_CLOSE_UPVALUE", offset),
            OP_GET_UPVALUE => self.simple_instruction("OP_GET_UPVALUE", offset),
            OP_SET_UPVALUE => self.simple_instruction("OP_SET_UPVALUE", offset),
            OP_CLASS => self.simple_instruction("OP_CLASS", offset),
            OP_GET_PROP => self.simple_instruction("OP_GET_PROP", offset),
            OP_SET_PROP => self.simple_instruction("OP_SET_PROP", offset),
            OP_METHOD => self.simple_instruction("OP_SET_PROP", offset),
            OP_INVOKE => self.simple_instruction("OP_INVOKE", offset),
            OP_INHERIT => self.simple_instruction("OP_INHERIT", offset),
            OP_SUPER_INVOKE => self.simple_instruction("OP_SUPER_INVOKE", offset),
            OP_GET_SUPER => self.simple_instruction("OP_SUPER_GET_PROP", offset),
            OP_INDEX_SET => self.simple_instruction("OP_INDEX_SET", offset),
            OP_CONSTANT0 => self.simple_instruction("OP_FOR", offset),
            OP_JUMP_IF_RANGE_END => self.simple_instruction("OP_JUMP_IF_RANGE_END", offset),
            OP_COUNTUP => self.simple_instruction("OP_COUNTUP", offset),
            OP_RANGE => self.simple_instruction("OP_RANGE", offset),
            _ => {
                println!("no match \"{:02X}\"", instruction);
                return offset + 1;
            }
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let index = match self.code.get(offset + 1) {
            Some(i) => match i.clone().try_into() {
                Ok(i) => i,
                Err(e) => panic!("{}", e),
            },
            None => panic!("out of index constant value."),
        };
        println!("{} {:?}", name, self.constants.get(index));
        return offset + 2;
    }

    fn jump_instruction(&self, name: &str, offset: usize) -> usize {
        let index: usize = match self.read_u16(offset + 1) {
            Some(c) => match c.try_into() {
                Ok(i) => i,
                Err(e) => panic!("{}", e),
            },
            None => panic!("out of index jump offset value."),
        };
        println!(
            "{} ({:04X} + {:04X} + {:04X} -> {:04X})",
            name,
            offset,
            3,
            index,
            offset + 3 + index
        );
        return offset + 3;
    }

    fn loop_instruction(&self, name: &str, offset: usize) -> usize {
        let index: usize = match self.read_u16(offset + 1) {
            Some(c) => match c.try_into() {
                Ok(i) => i,
                Err(e) => panic!("{}", e),
            },
            None => panic!("out of index jump offset value."),
        };
        println!(
            "{} ({:04X} + {:04X} - {:04X} -> {:04X})",
            name,
            offset,
            3,
            index,
            offset + 3 - index
        );
        return offset + 3;
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        return offset + 1;
    }
}
