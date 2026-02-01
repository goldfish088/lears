use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

mod vm;
use crate::vm::VM;

mod compiler;
use crate::compiler::Compiler;

mod scanner;

mod chunk;
use crate::chunk::Chunk;

mod list;

mod common;
use crate::common::{InterpretError, Value};

mod util;

fn run(code: String) -> Result<(), InterpretError> {
    let compiler = Compiler::new();
    let mut chunk = Chunk::<Value>::new("chunk".to_owned());

    if !compiler.compile(&mut chunk, code) {
        return Err(InterpretError::Compile);
    }

    let mut vm = VM::new();
    vm.interpret(&mut chunk)
}

fn run_file(path: &String) {
    let code = fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("oops something went wrong");
        String::new()
    });

    if let Err(err) = run(code) {
        eprintln!("{:?}", err);
    }
}

fn run_repl() {
    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        let n = io::stdin().read_line(&mut line).unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            0
        });

        if n == 0 {
            break;
        }

        if let Err(err) = run(line) {
            eprintln!("{:?}", err);
        }
    }
}

fn rlox_main() {
    let num_args = env::args().len();
    if num_args > 2 {
        let fullpath = env::args().next().unwrap();

        // NOTE: only UNIX compatible...
        println!(
            "Usage: ./{} [script]",
            match fullpath.rfind('/') {
                Some(i) => String::from(&fullpath[i + 1..]),
                _ => fullpath,
            }
        );

        process::exit(64);
    } else if num_args == 2 {
        let file = env::args().nth(1).unwrap();
        run_file(&file);
    } else {
        run_repl();
    }
}

fn debug_main() {
    use crate::common::OpCode::*;
    let mut chunk = Chunk::new("my first bytecode!".to_owned());

    // for i in 1..=10 {
    //     chunk.write_byte(Constant as u8, 123);
    //     let const_lookup = chunk.add_constant(Value::try_from(i).expect("should be ok")) as u8;
    //     chunk.write_byte(const_lookup, 123);
    // }

    // chunk.write_byte(Negate as u8, 123);

    {
        // testing add operation
        let one = chunk.add_constant(2.2);
        chunk.write_byte(Constant as u8, 123);
        chunk.write_byte(one as u8, 123);

        let mut two = chunk.add_constant(3.4);
        chunk.write_byte(Constant as u8, 123);
        chunk.write_byte(two as u8, 123);

        chunk.write_byte(Add as u8, 123);

        two = chunk.add_constant(5.6);
        chunk.write_byte(Constant as u8, 123);
        chunk.write_byte(two as u8, 123);

        chunk.write_byte(Divide as u8, 123);

        chunk.write_byte(Negate as u8, 123);
    }

    chunk.write_byte(Ret as u8, 123);
    println!("{}", &chunk);

    let mut vm = VM::new();
    match vm.interpret(&mut chunk) {
        Ok(_) => {}
        Err(err) => println!("{:?}", err),
    }
}

fn main() {
    // debug_main();
    rlox_main();
}
