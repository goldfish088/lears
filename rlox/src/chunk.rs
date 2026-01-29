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

pub struct Chunk<'a> {
    name: &'a str,
    bytecode: Vec<u8>,
}

impl<'a> Chunk<'a> {
    pub fn new(name: &'a str) -> Self {
        Chunk {
            name,
            bytecode: Vec::new(),
        }
    }
}

type FnInstrFmtAndLength = fn(opcode: OpCode) -> (String, usize);

impl fmt::Display for Chunk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let Chunk { name, bytecode, .. } = self;

        let simple_instruction: FnInstrFmtAndLength = |opcode| (format!("{}\n", opcode), 1);

        use OpCode::*;
        writeln!(f, "-- {} --", name)?;

        let mut offset = 0;
        while offset < bytecode.len() {
            offset += match OpCode::try_from(bytecode[offset]) {
                Ok(opcode) => {
                    write!(f, "{:04} ", offset)?;
                    match opcode {
                        Ret => {
                            let (fmt_opcode, length) = simple_instruction(opcode);
                            write!(f, "{}", fmt_opcode)?;
                            length
                        }
                        _ => unreachable!(),
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
