use std::fmt::{Display, Error, Formatter};

// Not including types you intend to use
// can cause great trouble if the names
// conflict with anything from the prelude.
use crate::list::List;

use crate::common::OpCode;

pub struct Chunk<V: Display> {
    name: String,
    bytecode: List<u8>,
    constants: List<V>,
    // TODO: change this to use run length encoding instead of storing the line for every
    // single byte
    lines: List<usize>,
}

impl<V: Display> Chunk<V> {
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

    pub fn get_constant(&self, lookup: usize) -> &V {
        &self.constants[lookup]
    }

    pub fn update_constant(&mut self, lookup: usize, replacement: V) {
        self.constants[lookup] = replacement;
    }

    pub fn add_constant(&mut self, value: V) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
impl<V: Display> Display for Chunk<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
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
                        Ret | Negate | Add | Subtract | Multiply | Divide => {
                            writeln!(f, "{}", opcode)?;
                            1
                        }
                        Constant => {
                            let lookup = bytecode[offset + 1] as usize;
                            writeln!(f, "{:<16} {:4} '{}'", opcode, lookup, &constants[lookup])?;
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
