use super::ast::ASTNode;
use super::bound::BoundChecker;
use super::lexer::{Lexer, LexerError};
use super::token::*;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, prelude::*};


pub struct Parser {
    i: usize,
    t_queue: VecDeque<Token>,
    lexer: Lexer,
    bound : BoundChecker,
}

pub struct ParserError {
    e: String,
    line: usize,
    col: usize,
}

impl Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.e)
    }
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
            i: 0,
            t_queue: VecDeque::new(),
            lexer: Lexer::new(contents, Some(filename)),
            bound : BoundChecker::new(),
        })
    }

    pub fn from_string(str: String) -> Self {
        Self {
            i: 0,
            t_queue: VecDeque::new(),
            lexer: Lexer::new(str, None),
            bound : BoundChecker::new(),
        }
    }

    pub fn bind(&mut self, name : String) {
        self.bound.add_binding(name);
    }

    fn node_id(&mut self) -> usize {
        self.i += 1;
        self.i
    }

    fn error(&self, msg: String) -> ParserError {
        ParserError {
            e: format!("error: [{}]: {}", self.lexer.pos_string(), msg),
            line: self.lexer.line,
            col: self.lexer.col,
        }
    }

    // Add tk to queue
    fn queue_tk(&mut self) -> Result<(), ParserError> {
        let t = self.lexer.get_token()?;
        self.t_queue.push_back(t);

        Ok(())
    }

    // Roll queue forwards
    fn advance(&mut self) {
        self.t_queue.pop_front();
    }

    // Get nth token without advancing
    fn peek(&mut self, n: usize) -> Result<Token, ParserError> {
        while n >= self.t_queue.len() {
            self.queue_tk()?;
        }
        Ok(self.t_queue[n].clone())
    }

    // Get 0th token and advance
    #[inline(always)]
    fn consume(&mut self) -> Result<Token, ParserError> {
        let peek_result = self.peek(0);
        self.advance();
        peek_result
    }

    fn parse_expression(&mut self) -> Result<ASTNode, ParserError> {
        let mut left = self.parse_primary()?;
        loop {
            match &self.peek(0)?.tt {
                // If paren, apply to paren
                TokenType::LParen => {
                    self.advance();
                    left = ASTNode::new_app(left, self.parse_expression()?, self.i)
                }
                TokenType::RParen | TokenType::EOF | TokenType::Newline => {
                    self.advance();
                    return Ok(left);
                }

                // If Primary
                TokenType::Id
                | TokenType::FloatLit
                | TokenType::CharLit
                | TokenType::IntLit
                | TokenType::StringLit => left = ASTNode::new_app(left, self.parse_primary()?, self.node_id()),
                
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
            TokenType::Id => {
                let id_name = t.value.clone();
                if !self.bound.is_bound(&id_name) {
                    return Err(self.error(format!("Unbound identifier: {}", id_name)));
                }
                Ok(ASTNode::new_id(t, self.node_id()))
            },
            TokenType::IntLit | TokenType::FloatLit => Ok(ASTNode::new_lit(t, self.node_id())),
            _ => Err(self.error(format!("Unexpected Token in primary: {:?}", t))),
        }
    }

    fn parse_assignment(&mut self) -> Result<ASTNode, ParserError> {
        assert_eq!(self.peek(0)?.tt, TokenType::Id);
        assert_eq!(self.peek(1)?.tt, TokenType::Assignment);

        let assid = self.peek(0)?;
        self.advance();
        self.advance();

        if self.bound.is_bound(&assid.value) {
            return Err(self.error(format!("Identifier already bound: {}", assid.value)));
        }

        Ok(ASTNode::new_assignment(assid, self.parse_expression()?, self.node_id()))
    }

    pub fn parse(&mut self) -> Result<ASTNode, ParserError> {
        // At the top level its just a set of assignments
        let mut ass_vec: Vec<ASTNode> = vec![];

        'assloop: loop {
            let t = self.peek(0)?;

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
                TokenType::Newline => {self.advance();}
                TokenType::EOF => {
                    break 'assloop;
                }
                _ => return Err(self.error(format!("Unexpected Token: {:?}", t))),
            }
        }

        Ok(ASTNode::new_module(ass_vec, self.node_id()))
    }
}
