use crate::chunk::Chunk;
use std::fmt::{Display, Error, Formatter};

use crate::list::List;

use crate::common::{OpCode, Value};

pub struct VM {
    stack: List<usize>,
}

#[derive(Debug)]
pub enum InterpretError {
    Compile,
    Runtime,
}

impl VM {
    pub fn new() -> Self {
        VM { stack: List::new() }
    }

    pub fn interpret(&mut self, chunk: &mut Chunk<Value>) -> Result<(), InterpretError> {
        use OpCode::*;
        let mut ip = 0;

        loop {
            let step = match OpCode::try_from(chunk.get_byte(ip)) {
                Err(_) => return Err(InterpretError::Runtime),
                Ok(Ret) => {
                    if let Some(chunk_idx) = self.stack.pop() {
                        println!("{}", chunk.get_constant(chunk_idx));
                    }
                    return Ok(());
                }
                Ok(Constant) => {
                    // TODO: this can potentially be out of bounds
                    let lookup = chunk.get_byte(ip + 1);
                    let value = chunk.get_constant(lookup as usize);
                    println!("{}", &value);
                    self.stack.push(lookup as usize);
                    2
                }
                Ok(Negate) => {
                    match self.stack.last() {
                        None => return Err(InterpretError::Runtime),
                        Some(chunk_idx) => {
                            let constant = *chunk.get_constant(*chunk_idx);
                            chunk.update_constant(*chunk_idx, -constant);
                        }
                    }
                    1
                }
            };

            ip += step;
        }
    }
}

impl Display for VM {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "          ")?;
        for i in 0..self.stack.len() {
            write!(f, "[ ")?;
            write!(f, "{}", &self.stack[i])?;
            write!(f, " ]")?
        }
        writeln!(f)
    }
}
