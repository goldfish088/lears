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
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Slash,
    SlashSlash,
    Whitespace,
    StringLiteral(String),
    NumberLiteral(f64),
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

    fn peek_char(&self) -> u8 {
        assert!(self.can_scan());
        self.source.as_bytes()[self.lex_curr_pos]
    }

    fn next_char(&mut self) -> u8 {
        assert!(self.can_scan());

        let c = self.source.as_bytes()[self.lex_curr_pos];
        self.lex_curr_pos += 1;
        c
    }

    fn lookahead_one(&mut self, val: u8, eq: Token, fallback: Token) -> Token {
        if !self.can_scan() {
            return fallback;
        }

        let c = self.source.as_bytes()[self.lex_curr_pos];
        if c == val {
            self.lex_curr_pos += 1;
            eq
        } else {
            fallback
        }
    }

    fn scan_token(&mut self) -> Result<Token, ScanError> {
        use Token::*;
        assert!(self.can_scan());

        let c = self.next_char();
        // TODO: implement an alternative matcher using a trie, keep this implementation
        // put all these methods under a trait, and this current approach can be one
        // such trait implementation.
        match c {
            // single char tokens
            b'(' => Ok(LParen),
            b')' => Ok(RParen),
            b'{' => Ok(LBrace),
            b'}' => Ok(RBrace),
            b',' => Ok(Comma),
            b'.' => Ok(Dot),
            b'-' => Ok(Minus),
            b'+' => Ok(Plus),
            b';' => Ok(Semicolon),
            b'*' => Ok(Star),

            // op tokens
            b'!' => Ok(self.lookahead_one(b'=', BangEqual, Bang)),
            b'=' => Ok(self.lookahead_one(b'=', EqualEqual, Equal)),
            b'<' => Ok(self.lookahead_one(b'=', LessEqual, Less)),
            b'>' => Ok(self.lookahead_one(b'=', GreaterEqual, Greater)),
            b'/' => Ok({
                if !self.can_scan() || self.peek_char() != b'/' {
                    Slash
                } else {
                    // TODO: is this ok?
                    // consumes the trailing newline due to `next_char` semantics
                    // so you will not a final Token::Whitespace in the token list
                    while self.can_scan() && self.next_char() != b'\n' {}
                    SlashSlash
                }
            }),
            b' ' | b'\t' | b'\r' => Ok(Whitespace),
            b'\n' => Ok({
                self.line += 1;
                Whitespace
            }),

            // string literals
            b'"' => Ok({
                let mut literal = Vec::<u8>::new();
                while self.can_scan() {
                    let nc = self.next_char();
                    if nc == b'"' {
                        break;
                    }
                    literal.push(nc);
                }

                StringLiteral(String::from_utf8(literal).unwrap())
            }),

            // number literals
            b'0'..=b'9' => Ok({
                let mut literal = vec![c];
                let mut seen_dot = false;
                while self.can_scan() {
                    let nc = self.next_char();
                    if nc == b'.' {
                        if seen_dot {
                            break;
                        }
                        seen_dot = true;
                    }

                    if !(b'0'..=b'9').contains(&nc) {
                        break;
                    }

                    literal.push(nc);
                }

                NumberLiteral(String::from_utf8(literal).unwrap().parse::<f64>().unwrap())
            }),
            _ => Err(ScanError {
                line: self.line,
                message: format!("Unexpected character '{}'.", char::from(c)),
            }),
        }
    }

    fn scan_tokens(&mut self) -> Result<Vec<Token>, Vec<ScanError>> {
        let mut tokens = Vec::<Token>::new();
        let mut errors = Vec::<ScanError>::new();

        while self.can_scan() {
            self.lex_start_pos = self.lex_curr_pos;
            match self.scan_token() {
                Ok(token) => {
                    tokens.push(token);

                    if !self.can_scan() {
                        tokens.push(Token::Eof);
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

        // let _ = line.pop();
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
