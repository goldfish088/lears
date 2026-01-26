use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

#[allow(dead_code)]
fn print_type<T>(_: &T) {
    println!("&type = {}", std::any::type_name::<&T>());
}

#[derive(Debug, PartialEq)]
enum Token {
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Eof,
    Dot,
    Minus,
    Semicolon,
    Plus,
    Star,
}

struct Scanner {
    source: String,

    line: usize,
    lex_start_pos: usize,
    lex_curr_pos: usize,
}

struct ScanError {
    line: usize,
    message: String,
}

impl ScanError {
    fn report(&self) {
        eprintln!("[line {}] Error: {}", self.line, self.message);
    }
}

impl Scanner {
    fn can_scan(&self) -> bool {
        self.lex_curr_pos < self.source.len()
    }

    fn next_char(&mut self) -> char {
        assert!(self.can_scan());

        let c = self
            .source
            .chars()
            .nth(self.lex_curr_pos)
            .unwrap_or_else(|| {
                eprintln!("could not get char at pos {}", self.lex_curr_pos);
                process::exit(65);
            });

        self.lex_curr_pos += 1;
        c
    }

    fn scan_token(&mut self) -> Result<Token, ScanError> {
        if !self.can_scan() {
            return Ok(Token::Eof);
        }

        let c = self.next_char();
        match c {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '{' => Ok(Token::LBrace),
            '}' => Ok(Token::RBrace),
            ',' => Ok(Token::Comma),
            '.' => Ok(Token::Dot),
            '-' => Ok(Token::Minus),
            '+' => Ok(Token::Plus),
            ';' => Ok(Token::Semicolon),
            '*' => Ok(Token::Star),
            _ => Err(ScanError {
                line: self.line,
                message: format!("Unexpected character '{}'.", c),
            }),
        }
    }

    fn scan_tokens(&mut self) -> Result<Vec<Token>, Vec<ScanError>> {
        let mut tokens = Vec::<Token>::new();
        let mut errors = Vec::<ScanError>::new();

        loop {
            self.lex_start_pos = self.lex_curr_pos;
            match self.scan_token() {
                Ok(token) => {
                    tokens.push(token);
                    if *tokens.last().unwrap() == Token::Eof {
                        break;
                    }
                }
                Err(error) => errors.push(error),
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }
}

fn interpret(code: String) {
    let mut scanner = Scanner {
        source: code,
        line: 1,
        lex_curr_pos: 0,
        lex_start_pos: 0,
    };

    match scanner.scan_tokens() {
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

    print_type(&code);
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

        let _ = line.pop();
        interpret(line);
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
