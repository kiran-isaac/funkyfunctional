use crate::parser::ast::ASTNodeType;

use super::ast::ASTNode;
use super::lexer::{Lexer, LexerError};
use super::token::*;
use std::cmp::{max, min};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, prelude::*};

pub struct Parser {
    t_queue: VecDeque<Token>,
    lexer: Lexer,
}

pub struct ParserError {
    e: String,
    line: usize,
    col: usize,
}

impl From<LexerError> for ParserError {
    fn from(value: LexerError) -> Self {
        Self {
            e: value.e,
            line: value.line,
            col: value.col,
        }
    }
}

impl Parser {
    pub fn from_file(filename: String) -> Result<Self, io::Error> {
        let mut file = File::open(&filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Self {
            t_queue: VecDeque::new(),
            lexer: Lexer::new(contents, Some(filename)),
        })
    }

    pub fn from_string(str: String) -> Self {
        Self {
            t_queue: VecDeque::new(),
            lexer: Lexer::new(str, None),
        }
    }

    fn error(&self, msg: String) -> ParserError {
        ParserError {
            e: format!("error: [{}]: {}", self.lexer.pos_string(), msg),
            line: self.lexer.line,
            col: self.lexer.col,
        }
    }

    fn queue_tk(&mut self) -> Result<(), ParserError> {
        let t = self.lexer.get_token()?;
        self.t_queue.push_back(t);

        Ok(())
    }

    // Roll queue forwards
    fn advance(&mut self) {
        self.t_queue.pop_front();
    }

    // Get token without advancing, with offset of 1
    fn peek(&mut self, i: usize) -> Result<Token, ParserError> {
        while i < self.t_queue.len() {
            self.queue_tk()?;
        }
        Ok(self.t_queue[i].clone())
    }

    #[inline(always)]
    fn consume(&mut self) -> Result<Token, ParserError> {
        self.advance();
        self.peek(0)
    }

    fn parse_expression(&mut self) -> Result<ASTNode, ParserError> {
        let mut left = self.parse_primary()?;
        loop {
            match &self.peek(0)?.tt {
                // If paren, apply to paren
                TokenType::LParen => {
                    self.advance();
                    left = ASTNode::new_app(left, self.parse_expression()?)
                }
                TokenType::RParen => {
                    self.advance();
                    return Ok(left);
                }

                // If Primary
                TokenType::Id
                | TokenType::FloatLit
                | TokenType::CharLit
                | TokenType::IntLit
                | TokenType::StringLit => left = ASTNode::new_app(left, self.parse_primary()?),

                _ => {
                    let e = format!("Unexpected token in expression: {:?}", self.peek(0)?);
                    return Err(self.error(e));
                }
            }
        }
    }

    fn parse_primary(&mut self) -> Result<ASTNode, ParserError> {
        let t = self.consume()?;
        match t.tt {
            TokenType::Id => Ok(ASTNode::new_id(t)),
            TokenType::IntLit | TokenType::FloatLit => Ok(ASTNode::new_lit(t)),
            _ => Err(self.error(format!("Unexpected Token: {:?}", t))),
        }
    }

    fn parse_assignment(&mut self) -> Result<ASTNode, ParserError> {
        assert_eq!(self.peek(0)?.tt, TokenType::Id);
        assert_eq!(self.peek(1)?.tt, TokenType::Assignment);

        let assid = self.peek(0)?;
        self.advance();

        Ok(ASTNode::new_assignment(assid, self.parse_expression()?))
    }

    pub fn parse(&mut self) -> Result<ASTNode, ParserError> {
        let t = self.peek(0)?;
        let mut ass_vec: Vec<ASTNode> = vec![];

        // At the top level its just a set of assignments

        'assloop : loop {
            match t.tt {
                TokenType::Id => match self.peek(1)?.tt {
                    TokenType::Assignment => ass_vec.push(self.parse_assignment()?),
                    _ => {
                        return Err(self.error(format!(
                            "Unexpected Token: {:?}. Expected assignment operator: {:?}",
                            t,
                            TokenType::Assignment
                        )))
                    }
                },
                TokenType::EOF => {
                    break 'assloop;
                }
                _ => return Err(self.error(format!("Unexpected Token: {:?}", t))),
            }
        }

        Ok(ASTNode::new_module(ass_vec))
    }
}
