use std::convert::TryFrom;
use std::fmt;

// Not including types you intend to use
// can cause great trouble if the names
// conflict with anything from the prelude.
use crate::list::List;

// Our instruction set

#[derive(fmt::Debug)]
pub enum OpCode {
    Ret,
    Constant,
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
            1 => Ok(Constant),
            _ => Err("Invalid opcode"),
        }
    }
}

// TODO: add more constant types like string literals
pub type Value = f64;

pub struct Chunk {
    name: String,
    bytecode: List<u8>,
    constants: List<Value>,
    // TODO: change this to use run length encoding instead of storing the line for every
    // single byte
    lines: List<usize>,
}

impl Chunk {
    pub fn new(name: String) -> Self {
        Chunk {
            name,
            bytecode: List::new(),
            constants: List::new(),
            lines: List::new(),
        }
    }

    pub fn get_byte(&self, offset: usize) -> u8 {
        self.bytecode[offset]
    }

    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.bytecode.push(byte);
        self.lines.push(line);
    }

    pub fn get_constant(&self, lookup: usize) -> Value {
        self.constants[lookup]
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let Chunk {
            name,
            bytecode,
            constants,
            lines,
            ..
        } = self;

        use OpCode::*;
        writeln!(f, "-- {} --", name)?;

        let mut offset = 0;
        while offset < bytecode.len() {
            offset += match OpCode::try_from(bytecode[offset]) {
                Ok(opcode) => {
                    write!(f, "{:04} ", offset)?;
                    if offset > 0 && lines[offset] == lines[offset - 1] {
                        write!(f, "   | ")?;
                    } else {
                        write!(f, "{:4} ", lines[offset])?;
                    }
                    match opcode {
                        Ret => {
                            writeln!(f, "{}", opcode)?;
                            1
                        }
                        Constant => {
                            let lookup = bytecode[offset + 1] as usize;
                            writeln!(f, "{:<16} {:4} '{}'", opcode, lookup, constants[lookup])?;
                            2
                        }
                    }
                }
                Err(error) => {
                    writeln!(f, "{}", error)?;
                    1
                }
            }
        }

        Ok(())
    }
}
