pub use super::token::*;

pub type LexerError = String;

pub struct Lexer {
    filename: String,
    file: Vec<char>,
    i: usize,
}

impl Lexer {
    pub fn new(filename: String, file: String) -> Self {
        let vec = file.chars().collect();
        Lexer {
            filename,
            file: vec,
            i: 0,
        }
    }

    #[inline(always)]
    fn c(&self) -> char {
        self.file[self.i]
    }

    #[inline(always)]
    fn skip_whitespace(&mut self) {
        while self.i < self.file.len() && self.file[self.i].is_whitespace() {
            self.i += 1;
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
            tt: TokenType::Identifier,
            value: str,
        });
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
