use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

mod scanner;

#[allow(dead_code)]
fn print_type<T>(_: &T) {
    println!("&type = {}", std::any::type_name::<&T>());
}

fn interpret(code: &str) {
    let mut scanner = scanner::Scanner::new(code);

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

    interpret(code.as_str());
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

        interpret(line.as_str());
    }
}

fn main() {
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
