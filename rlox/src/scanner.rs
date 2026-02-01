// handwritten scanner/lexer for the lox syntax grammar

use crate::list::List;

use crate::common::Token;

#[derive(Debug)]
pub struct Scanner {
    source: String,

    line: usize,
    lex_start_pos: usize,
    lex_curr_pos: usize,
}

pub struct ScanError {
    line: usize,
    message: String,
}

impl ScanError {
    pub fn report(&self) {
        eprintln!("[line {}] Error: {}", self.line, self.message);
    }
}

impl Scanner {
    pub fn new(code: String) -> Self {
        Scanner {
            source: code,
            line: 1,
            lex_curr_pos: 0,
            lex_start_pos: 0,
        }
    }

    fn can_scan(&self) -> bool {
        self.lex_curr_pos < self.source.len()
    }

    fn peek_next(&self) -> u8 {
        assert!(self.can_scan());
        self.source.as_bytes()[self.lex_curr_pos]
    }

    fn scan_next(&mut self) -> u8 {
        assert!(self.can_scan());

        let c = self.source.as_bytes()[self.lex_curr_pos];
        self.lex_curr_pos += 1;
        c
    }

    fn scan_next_if(&mut self, val: u8, eq: Token, fallback: Token) -> Token {
        if !self.can_scan() {
            return fallback;
        }

        let c = self.peek_next();
        if c == val {
            self.lex_curr_pos += 1;
            eq
        } else {
            fallback
        }
    }

    fn skip_whitespace(&mut self) {
        if !self.can_scan() {
            return;
        }
        let mut advance = true;
        while advance {
            let c = self.peek_next();
            advance = match c {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.scan_next();

                    if c == b'\n' {
                        self.line += 1;
                    }

                    self.can_scan()
                }
                _ => false,
            };
        }
    }

    pub fn emit_next(&mut self) -> Result<Token, ScanError> {
        use Token::*;
        self.skip_whitespace();
        if !self.can_scan() {
            return Ok(Eof);
        }

        let c = self.scan_next();
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
            b'!' => Ok(self.scan_next_if(b'=', BangEqual, Bang)),
            b'=' => Ok(self.scan_next_if(b'=', EqualEqual, Equal)),
            b'<' => Ok(self.scan_next_if(b'=', LessEqual, Less)),
            b'>' => Ok(self.scan_next_if(b'=', GreaterEqual, Greater)),
            b'/' => Ok({
                if !self.can_scan() || self.peek_next() != b'/' {
                    Slash
                } else {
                    while self.can_scan() && self.peek_next() != b'\n' {
                        let _ = self.scan_next();
                    }
                    SlashSlash
                }
            }),

            // string literals
            // TODO: multiline does not work in REPL mode
            b'"' => {
                let mut literal = List::<u8>::new();
                while self.can_scan() {
                    let nc = self.scan_next();
                    literal.push(nc);

                    if nc == b'\n' {
                        self.line += 1;
                    } else if nc == b'"' {
                        break;
                    }
                }

                if literal.is_empty() || *literal.last().unwrap() != b'"' {
                    Err(ScanError {
                        line: self.line,
                        message: format!(
                            "Unterminated string literal \"{}",
                            str::from_utf8(&literal as &[u8]).unwrap()
                        ),
                    })
                } else {
                    let _ = literal.pop();
                    Ok(LiteralString(
                        str::from_utf8(&literal as &[u8]).unwrap().to_owned(),
                    ))
                }
            }

            // number literals
            b'0'..=b'9' => Ok({
                let mut literal = vec![c];
                let mut seen_dot = false;
                while self.can_scan() {
                    let nc = self.peek_next();

                    if !nc.is_ascii_digit() && nc != b'.' {
                        break;
                    }

                    if nc == b'.' {
                        if seen_dot {
                            break;
                        }
                        seen_dot = true;
                    }

                    let _ = self.scan_next();
                    literal.push(nc);
                }

                LiteralNumber(String::from_utf8(literal).unwrap().parse::<f64>().unwrap())
            }),

            c if (c.is_ascii_alphanumeric() || c == b'_') => Ok({
                let mut identifier = vec![c];
                while self.can_scan() {
                    let nc = self.peek_next();
                    if !nc.is_ascii_alphanumeric() && nc != b'_' {
                        break;
                    }

                    let _ = self.scan_next();
                    identifier.push(nc);
                }

                let raw_identifier = String::from_utf8(identifier).unwrap();
                match raw_identifier.as_str() {
                    "and" => KeywordAnd,
                    "class" => KeywordClass,
                    "else" => KeywordElse,
                    "false" => KeywordFalse,
                    "fun" => KeywordFun,
                    "for" => KeywordFor,
                    "if" => KeywordIf,
                    "nil" => KeywordNil,
                    "or" => KeywordOr,
                    "print" => KeywordPrint,
                    "return" => KeywordReturn,
                    "super" => KeywordSuper,
                    "this" => KeywordThis,
                    "true" => KeywordTrue,
                    "var" => KeywordVar,
                    "while" => KeywordWhile,
                    _ => Identifier(raw_identifier),
                }
            }),
            _ => Err(ScanError {
                line: self.line,
                message: format!("Unexpected character '{}'", char::from(c)),
            }),
        }
    }
}
