use super::lexer::{Lexer, LexerError};
use super::token::*;
use std::fs::File;
use std::io::{self, prelude::*};

pub struct Parser {
    t : Option<Token>,
    lexer : Lexer,
}

pub struct ParserError {
    e : String,
    line : usize,
    col : usize,
}

impl From<LexerError> for ParserError {
    fn from(value: LexerError) -> Self {
        Self {e : value.e, line: value.line, col: value.col}
    }
}

impl Parser {
    pub fn from_file(filename : String) -> Result<Self, io::Error> {
        let mut file = File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Self::from_string(contents))
    }

    pub fn from_string(str : String) -> Self {
        Self { t : None, lexer : Lexer::new(str) }
    }

    fn error(&self, msg : String) -> ParserError {
        ParserError {e: msg, line : self.lexer.line, col : self.lexer.col}
    }
}