use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

mod scanner;
use crate::scanner::Scanner;

mod chunk;
use crate::chunk::Chunk;

mod list;

mod util;

fn interpret(code: String) {
    let mut scanner = Scanner::new(code);

    match scanner.emit_all() {
        Ok(tokens) => {
            for token in tokens {
                println!("{:#?}", token);
            }
        }
        Err(errors) => {
            for error in errors {
                error.report();
            }
        }
    }
}

fn run_file(path: &String) {
    let code = fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("oops something went wrong");
        String::new()
    });

    interpret(code);
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

        interpret(line);
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

fn main() {
    use crate::chunk::OpCode::*;
    {
        let mut chunk = Chunk::new("my first bytecode!".to_owned());

        for i in 1..=10 {
            chunk.write_byte(Constant as u8, 123);
            let const_lookup =
                chunk.add_constant(f64::try_from(i).expect("should be ok") + 0.42) as u8;
            chunk.write_byte(const_lookup, 123);
        }

        chunk.write_byte(Ret as u8, 124);
        println!("{}", &chunk);
    }
    rlox_main();
}
