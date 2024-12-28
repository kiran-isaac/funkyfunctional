use crate::types::TypeError;

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
    bound: BoundChecker,
    ast: AST,
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
            bound: BoundChecker::new(),
            ast: AST::new(),
        })
    }

    pub fn from_string(str: String) -> Self {
        Self {
            i: 0,
            t_queue: VecDeque::new(),
            lexer: Lexer::new(str, None),
            bound: BoundChecker::new(),
            ast: AST::new(),
        }
    }

    pub fn add_bindings_from(&mut self, other: &Parser) {
        self.bound.append(&other.bound);
    }

    pub fn bind(&mut self, name: String) {
        self.bound.add_binding(name);
    }

    /// Used when it doesnt matter that something is already 
    /// bound, like when we are binding a local variable in 
    /// a lambda
    /// This will create an alias for the bound variable
    /// and return the alias
    pub fn bind_local(&mut self, name: String) -> String {
        let mut alias_id = 0; 
        while self.bound.is_bound(name.as_str()) {
            alias_id += 1;
        }

        if alias_id == 0 {
            self.bound.add_binding(name.clone());
            return name;
        } else {
            let alias = format!("{}{}", alias_id, name);
            self.bound.add_binding(alias.clone());
            return alias;
        }
    }

    pub fn unbind(&mut self, name: String) {
        self.bound.remove_binding(name);
    }

    fn parse_error(&self, msg: String) -> ParserError {
        ParserError {
            e: format!("parse error: [{}]: {}", self.lexer.pos_string(), msg),
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

    fn parse_abstraction(&mut self, ast: &mut AST) -> Result<usize, ParserError> {
        match self.peek(0)?.tt {
            TokenType::Id => {
                let id = self.consume()?;
                let varname = id.value.clone();
                if self.bound.is_bound(&varname) {
                    return Err(self.parse_error(format!("Identifier already bound, so cannot be bound for lambda: {}", varname)));
                }
                self.bind(varname.clone());
                let line = self.lexer.line;
                let col = self.lexer.col;
                let id = ast.add_id(id, line, col);
                match self.peek(0)?.tt {
                    TokenType::Dot => {
                        self.advance();
                        let expr = self.parse_expression(ast)?;
                        self.unbind(varname);
                        Ok(ast.add_abstraction(id, expr, line, col))
                    }
                    TokenType::Id => {
                        let abst2 = self.parse_abstraction(ast)?;
                        self.unbind(varname);
                        Ok(ast.add_abstraction(id, abst2, line, col))
                    }
                    _ => Err(self.parse_error("Expected dot after lambda id".to_string())),
                }
            }
            // if no arg we can just create a fake id that nothing can match
            TokenType::Dot => {
                let line = self.lexer.line;
                let col = self.lexer.col;
                let body = self.parse_expression(ast)?;
                // Its impossible to define a variable with a null name
                // so this is a safe fake id and wont be matched
                let fake_id = Token {
                    tt: TokenType::Id,
                    value: "".to_string(),
                };
                let id = ast.add_id(fake_id, line, col);
                Ok(ast.add_abstraction(id, body, line, col))
            }
            _ => Err(self.parse_error("Expected identifier after lambda".to_string())),
        }
    }

    fn parse_expression(&mut self, ast: &mut AST) -> Result<usize, ParserError> {
        let line = self.lexer.line;
        let col = self.lexer.col;
        let mut left = self.parse_primary(ast)?;
        loop {
            match &self.peek(0)?.tt {
                // If paren, apply to paren
                TokenType::LParen => {
                    self.advance();
                    let right = self.parse_expression(ast)?;
                    self.advance();
                    left = ast.add_app(left, right, line, col);
                }
                TokenType::RParen | TokenType::EOF | TokenType::Newline => {
                    return Ok(left);
                }

                TokenType::Lambda => {
                    self.advance();
                    self.parse_abstraction(ast)?;
                }

                // If Primary
                TokenType::Id
                | TokenType::FloatLit
                | TokenType::CharLit
                | TokenType::IntLit
                | TokenType::BoolLit => {
                    let right = self.parse_primary(ast)?;
                    left = ast.add_app(left, right, line, col);
                }

                _ => {
                    let e = format!("Unexpected token in expression: {:?}", self.peek(0)?);
                    return Err(self.parse_error(e));
                }
            }
        }
    }

    // Parse a primary expression
    fn parse_primary(&mut self, ast: &mut AST) -> Result<usize, ParserError> {
        let line = self.lexer.line;
        let col = self.lexer.col;
        let t = self.consume()?;
        match t.tt {
            TokenType::Id => {
                let id_name = t.value.clone();
                if !self.bound.is_bound(&id_name) {
                    return Err(self.parse_error(format!("Unbound identifier: {}", id_name)));
                }
                Ok(ast.add_id(t, line, col))
            }
            TokenType::IntLit | TokenType::FloatLit | TokenType::BoolLit | TokenType::CharLit => {
                Ok(ast.add_lit(t, line, col))
            }
            TokenType::Lambda => {
                self.advance();
                self.parse_abstraction(ast)
            }
            TokenType::LParen => {
                let exp = self.parse_expression(ast)?;
                self.advance();
                Ok(exp)
            }
            _ => Err(self.parse_error(format!("Unexpected Token in primary: {:?}", t))),
        }
    }

    fn parse_assignment(&mut self, ast: &mut AST) -> Result<usize, ParserError> {
        assert_eq!(self.peek(0)?.tt, TokenType::Id);
        assert_eq!(self.peek(1)?.tt, TokenType::Assignment);

        let assid = self.peek(0)?;
        self.advance();
        self.advance();

        if self.bound.is_bound(&assid.value) {
            return Err(self.parse_error(format!("Identifier already bound: {}", assid.value)));
        }

        let line = self.lexer.line;
        let col = self.lexer.col;

        self.bind(assid.value.clone());
        let exp = self.parse_expression(ast)?;
        let id = ast.add_id(assid, line, col);

        Ok(ast.add_assignment(id, exp, line, col))
    }

    pub fn parse_module(&mut self) -> Result<AST, ParserError> {
        // At the top level its just a set of assignments
        let mut ast = AST::new();
        let module = ast.add_module(Vec::new(), self.lexer.line, self.lexer.col);
        let mut main_found = false;

        'assloop: loop {
            let t = self.peek(0)?;

            match t.tt {
                TokenType::Id => match self.peek(1)?.tt {
                    TokenType::Assignment => {
                        if main_found {
                            return Err(self.parse_error(
                                "Main should be the last assignment in the module".to_string(),
                            ));
                        }

                        let assignment = self.parse_assignment(&mut ast)?;
                        ast.add_to_module(module, assignment);

                        if ast.get_assignee(assignment) == "main" {
                            main_found = true;
                        }
                    }
                    _ => {
                        return Err(self.parse_error(format!(
                            "Unexpected Token: {:?}. Expected assignment operator: {:?}",
                            t,
                            TokenType::Assignment
                        )))
                    }
                },
                TokenType::Newline => {
                    self.advance();
                }
                TokenType::EOF => {
                    break 'assloop;
                }
                _ => return Err(self.parse_error(format!("Unexpected Token: {:?}", t))),
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
