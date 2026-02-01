use std::fmt::{Display, Error, Formatter};

// TODO: add more constant types like string literals
pub type Value = f64;

// Our instruction set

#[derive(Debug)]
pub enum OpCode {
    Ret,
    Constant,
    Negate,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
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
            2 => Ok(Negate),
            _ => Err("Invalid opcode"),
        }
    }
}
