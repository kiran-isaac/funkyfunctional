pub use super::token::*;

pub struct LexerError {
    pub e: String,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    file: Vec<char>,
    filename: Option<String>,
    i: usize,
    pub line: usize,
    pub col: usize,
}

impl Lexer {
    pub fn new(file: String, filename: Option<String>) -> Self {
        let vec = file.chars().collect();
        Lexer {
            file: vec,
            filename,
            i: 0,
            line: 0,
            col: 0,
        }
    }

    #[inline(always)]
    fn c(&self) -> char {
        self.file[self.i]
    }

    fn advance(&mut self) {
        self.col += 1;
        self.i += 1;
    }

    fn error(&self, msg: String) -> LexerError {
        LexerError {
            e: format!("error: [{}]: {}", self.pos_string(), msg),
            line: self.line,
            col: self.col,
        }
    }

    #[inline(always)]
    pub fn pos_string(&self) -> String {
        format!(
            "{}:{}/{}",
            match &self.filename {
                None => "".to_string(),
                Some(f) => f.clone(),
            },
            self.line,
            self.col
        )
    }

    fn skip_whitespace(&mut self) {
        while self.i < self.file.len() && self.file[self.i].is_whitespace() {
            if self.c() == '\n' {
                self.line += 1;
                self.col = 0;
                self.i += 1;
            } else {
                self.advance();
            }
        }
    }

    fn parse_word(&mut self) -> Result<Token, LexerError> {
        let mut str = self.c().to_string();

        self.i += 1;

        while !self.c().is_whitespace() {
            match self.c() {
                'a'..'z' | 'A'..'Z' | '0'..'9' | '_' => {}

                _ => {}
            };
            str.push(self.c());
            self.i += 1;
        }

        return Ok(Token {
            tt: TokenType::Id,
            value: str,
        });
    }

    fn parse_num_lit(&mut self) -> Result<Token, LexerError> {
        let mut str = String::new();

        let mut has_point = false;

        while !self.c().is_whitespace() {
            match self.c() {
                '0'..'9' => {
                    str.push(self.c());
                }
                '.' => {
                    if has_point {
                        return Err(self.error(format!("Unexpected char: {}", self.c())));
                    }
                    has_point = true;
                    str.push(self.c());
                }
                _ => return Err(self.error(format!("Unexpected char in char literal: {}", self.c()))),
            }

            self.advance();
        }

        if has_point {
            return Ok(Token {
                tt: TokenType::FloatLit,
                value: str,
            });
        } else {
            return Ok(Token {
                tt: TokenType::IntLit,
                value: str,
            });
        }
    }

    pub fn get_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace();

        match self.c() {
            'a'..'z' => self.parse_word(),
            '(' => Ok(Token {
                tt: TokenType::RParen,
                value: "(".to_string(),
            }),
            ')' => Ok(Token {
                tt: TokenType::RParen,
                value: ")".to_string(),
            }),
            '=' => Ok(Token {
                tt: TokenType::Assignment,
                value: "=".to_string(),
            }),
            _ => unreachable!(),
        }
    }
}
