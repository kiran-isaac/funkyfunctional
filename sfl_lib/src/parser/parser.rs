use super::ast::{ASTNode, AST};
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
    ast : AST
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
            ast : AST::new()
        })
    }

    pub fn from_string(str: String) -> Self {
        Self {
            i: 0,
            t_queue: VecDeque::new(),
            lexer: Lexer::new(str, None),
            bound : BoundChecker::new(),
            ast : AST::new()
        }
    }

    pub fn add_bindings_from(&mut self, other : &Parser) {
        self.bound.append(&other.bound);
    }

    pub fn bind(&mut self, name : String) {
        self.bound.add_binding(name);
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

    fn parse_expression(&mut self, ast : &mut AST) -> Result<usize, ParserError> {
        let mut left = self.parse_primary(ast)?;
        loop {
            match &self.peek(0)?.tt {
                // If paren, apply to paren
                TokenType::LParen => {
                    self.advance();
                    let right = self.parse_expression(ast)?;
                    left = ast.add_app(left, right);
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
                | TokenType::StringLit => {
                    let right = self.parse_primary(ast)?;
                    left = ast.add_app(left, right);
                }
                
                _ => {
                    let e = format!("Unexpected token in expression: {:?}", self.peek(0)?);
                    return Err(self.error(e));
                }
            }
        }
    }

    fn parse_primary(&mut self, ast : &mut AST) -> Result<usize, ParserError> {
        let t = self.consume()?;
        match t.tt {
            TokenType::Id => {
                let id_name = t.value.clone();
                if !self.bound.is_bound(&id_name) {
                    return Err(self.error(format!("Unbound identifier: {}", id_name)));
                }
                Ok(ast.add_id(t))
            },
            TokenType::IntLit | TokenType::FloatLit => Ok(ast.add_lit(t)),
            _ => Err(self.error(format!("Unexpected Token in primary: {:?}", t))),
        }
    }

    fn parse_assignment(&mut self, ast : &mut AST) -> Result<usize, ParserError> {
        assert_eq!(self.peek(0)?.tt, TokenType::Id);
        assert_eq!(self.peek(1)?.tt, TokenType::Assignment);

        let assid = self.peek(0)?;
        self.advance();
        self.advance();

        if self.bound.is_bound(&assid.value) {
            return Err(self.error(format!("Identifier already bound: {}", assid.value)));
        }

        self.bind(assid.value.clone());
        let exp = self.parse_expression(ast)?;
        let id = ast.add_id(assid);

        Ok(ast.new_assignment(id, exp))
    }

    pub fn parse_module(&mut self) -> Result<AST, ParserError> {
        // At the top level its just a set of assignments
        let mut ast = AST::new();
        let module = ast.add_module(Vec::new());

        'assloop: loop {
            let t = self.peek(0)?;

            match t.tt {
                TokenType::Id => match self.peek(1)?.tt {
                    TokenType::Assignment => {
                        let assignment = self.parse_assignment(&mut ast)?;
                        ast.add_to_module(module, assignment);
                    }
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

        Ok(ast)
    }

    pub fn parse_tl_expression(&mut self) -> Result<AST, ParserError> {
        let mut ast = AST::new();
        ast.root = self.parse_expression(&mut ast)?;
        Ok(ast)
    }
}
