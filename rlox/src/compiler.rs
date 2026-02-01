use crate::chunk::Chunk;
use crate::common::{OpCode, Token, Value};
use crate::scanner::Scanner;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }

    fn write_constant(&self, chunk: &mut Chunk<Value>, constant: Value, line: usize) {
        let i = chunk.add_constant(constant);
        if i > u8::MAX.into() {
            eprintln!("No more room in constant pool!");
            return;
        }

        chunk.write_byte(OpCode::Constant as u8, line);
        chunk.write_byte(i as u8, line);
    }

    pub fn compile_expression(&self, chunk: &mut Chunk<Value>) {}

    pub fn compile(&self, chunk: &mut Chunk<Value>, code: String) -> bool {
        let mut scanner = Scanner::new(code);

        let mut already_err = false;

        loop {
            match scanner.emit_next() {
                Ok(token) => {
                    // translate token into bytecode stored in `self`
                    match token {
                        Token::LiteralNumber(num) => {
                            self.write_constant(chunk, num, scanner.get_line_number())
                        }
                        Token::Minus => {
                            self.compile_expression(chunk);
                            chunk.write_byte(OpCode::Negate as u8, scanner.get_line_number());
                        }
                        Token::LParen => {
                            self.compile_expression(chunk);
                            if scanner.scan_next_if(b')', Token::RParen, Token::Eof) == Token::Eof {
                                eprintln!("Expected ')' after expression.");
                                already_err = true;
                            }
                        }
                        _ => todo!(),
                    }

                    if token == Token::Eof {
                        break;
                    }
                }
                Err(err) => {
                    if !already_err {
                        err.report()
                    } else {
                        already_err = true;
                    }
                }
            }
        }

        false
    }
}
