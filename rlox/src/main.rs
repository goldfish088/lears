use std::env;
use std::process;
use std::fs;
use std::io::{self, Write};
use std::vec;

#[allow(dead_code)]
fn print_type<T>(_: &T) {
    println!("&type = {}", std::any::type_name::<&T>());
}

struct Token {

}

fn scan_tokens() -> Vec<Token> {
    Vec::new::<>()
}

fn interpret(code: &str) {
    println!("length = {}", code.len());
    println!("code = {}", code);
    todo!();
}

fn run_file(path: &String) {
    // if code.is_err() {
    //     eprintln!("oops something went wrong");
    //     process::exit(42);
    // }
    let code = fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("oops something went wrong");
        process::exit(42);
    });

    print_type(&code);
    interpret(&code);
}

fn run_repl() {
    // init scanner, and other additional state

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

        line.trim();
        interpret(&line);
    }
}

fn main() {
    let num_args = env::args().len();
    if num_args > 2 {
        let fullpath = env::args().next().unwrap();

        // NOTE: only UNIX compatible...
        println!("Usage: ./{} [script]", match fullpath.rfind('/') {
            Some(i) => String::from(&fullpath[i+1..]),
            _ => fullpath
        });

        process::exit(64);
    } else if num_args == 2 {
        let file = env::args().nth(1).unwrap();
        run_file(&file);
    } else {
        run_repl();
    }
}
