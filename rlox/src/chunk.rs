use std::convert::TryFrom;
use std::fmt;
use std::ops::{Deref, DerefMut};

// Our instruction set

#[derive(fmt::Debug)]
enum OpCode {
    Ret,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<u8> for OpCode {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use OpCode::*;
        match value {
            0 => Ok(Ret),
            _ => Err("Invalid opcode"),
        }
    }
}

pub struct Chunk<'_> {
    name: &'_ str,
    bytecode: Vec<u8>,
}

impl Chunk {
    pub fn new(name: &str) -> Self {
        Chunk {
            name,
            bytecode: Vec::new(),
        }
    }
}

// TODO: implement me with closures
impl fmt::Display for Chunk<'_> {
    fn simple_instruction(opcode: OpCode, offset: usize) -> usize {
        println!("{}", opcode);
        offset + 1
    }

    /*
     Returns the offset in the Chunk bytecode corresponding
     to one after the "end" of the *current* instruction we
     disassembled starting at `offset`

     It is also the start of the *next* instruction
    */
    fn disassemble_at(&self, offset: usize) -> usize {
        use OpCode::*;
        let Chunk { bytecode, .. } = self;
        match OpCode::try_from(bytecode[offset]) {
            Ok(opcode) => {
                print!("{:04} ", offset);
                match opcode {
                    Ret => Self::simple_instruction(opcode, offset),
                    _ => unreachable!(),
                }
            }
            Err(error) => {
                println!("{}", error);
                offset + 1
            }
        }
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let Chunk { name, bytecode } = self;
        writeln!("-- {} --", name);

        let mut offset = 0;
        while offset < bytecode.len() {
            offset = self.disassemble_at(offset);
        }
    }
}

// Forward Vec<T> methods to Chunk

impl Deref for Chunk<'_> {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bytecode
    }
}

impl DerefMut for Chunk<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytecode
    }
}
