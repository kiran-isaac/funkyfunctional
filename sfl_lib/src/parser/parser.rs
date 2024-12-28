use super::ast::AST;
use super::bound::BoundChecker;
use super::lexer::{Lexer, LexerError};
use super::token::*;
use crate::{Primitive, Type};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, prelude::*};

pub struct Parser {
    t_queue: VecDeque<Token>,
    lexer: Lexer,
    bound: BoundChecker,
    type_assignment_map: HashMap<String, Type>,
}

pub struct ParserError {
    e: String,
    line: usize,
    col: usize,
}

impl Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Parser Error at [{}:{}]: {}",
            self.line, self.col, self.e
        )
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
            t_queue: VecDeque::new(),
            lexer: Lexer::new(contents, Some(filename)),
            bound: BoundChecker::new(),
            type_assignment_map: HashMap::new(),
        })
    }

    pub fn from_string(str: String) -> Self {
        Self {
            t_queue: VecDeque::new(),
            lexer: Lexer::new(str, None),
            bound: BoundChecker::new(),
            type_assignment_map: HashMap::new(),
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
            e: msg,
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
                if varname != "_" {
                    if self.bound.is_bound(&varname) {
                        return Err(self.parse_error(format!(
                            "Identifier already bound, so cannot be bound for lambda: {}",
                            varname
                        )));
                    }
                    self.bind(varname.clone());
                }
                let line = self.lexer.line;
                let col = self.lexer.col;
                let id = ast.add_id(id, line, col);
                match self.peek(0)?.tt {
                    TokenType::Dot => {
                        self.advance();
                        let expr = match self.peek(0)?.tt {
                            TokenType::Lambda => {
                                self.advance();
                                self.parse_abstraction(ast)?
                            }
                            _ => self.parse_expression(ast)?,
                        };
                        if varname != "_" {
                            self.unbind(varname);
                        }
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
            _ => Err(self.parse_error("Expected identifier (or ignore directive '_') after lambda".to_string())),
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
            // Removed support for lambda except at the top level
            // for now, untill i figure out type inference
            // TokenType::Lambda => {
            //     self.advance();
            //     self.parse_abstraction(ast)
            // }
            TokenType::LParen => {
                let exp = self.parse_expression(ast)?;
                self.advance();
                Ok(exp)
            }
            _ => Err(self.parse_error(format!("Unexpected Token in primary: {:?}", t))),
        }
    }

    fn parse_type_expression(&mut self, ast: &mut AST) -> Result<Type, ParserError> {
        let mut left = self.parse_type_expression_primary(ast)?;

        loop {
            let next = self.peek(0)?;
            let left_string = left.to_string();

            match next.tt {
                TokenType::RArrow => {
                    self.advance();
                    let right = self.parse_type_expression(ast)?;
                    let right_string = right.to_string();
                    left = Type::Function(Box::new(left), Box::new(right));
                }
                TokenType::RParen | TokenType::Newline | TokenType::EOF => return Ok(left),
                _ => {
                    return Err(self
                        .parse_error(format!("Unexpected token in type expression: {:?}", next)))
                }
            }
        }
    }

    fn parse_type_expression_primary(&mut self, ast: &mut AST) -> Result<Type, ParserError> {
        let t = self.consume()?;

        match t.tt {
            TokenType::TypeId => {
                let id = t.value;
                match id.as_str() {
                    "Int" => Ok(Type::Primitive(Primitive::Int64)),
                    "Float" => Ok(Type::Primitive(Primitive::Float64)),
                    _ => unimplemented!("Only Int and Float are supported"),
                }
            }
            TokenType::LParen => {
                let inner = self.parse_type_expression(ast)?;
                let inner_string = inner.to_string();
                self.advance();
                Ok(inner)
            }
            _ => Err(self.parse_error(format!("Unexpected token in type expression: {:?}", t))),
        }
    }

    fn parse_type_assignment(&mut self, ast: &mut AST) -> Result<(), ParserError> {
        let name = self.peek(0)?.value.clone();
        if self.type_assignment_map.contains_key(&name) {
            return Err(self.parse_error(format!("Type already assigned: {}", name)));
        }
        self.advance();
        self.advance();

        let assigned_type = self.parse_type_expression(ast)?;
        self.type_assignment_map.insert(name, assigned_type);

        Ok(())
    }

    pub fn get_type_assignment(&self, name: &String) -> Result<Type, ParserError> {
        match self.type_assignment_map.get(name) {
            Some(t) => Ok(t.clone()),
            None => Err(self.parse_error(format!("Type not assigned: {}", name))),
        }
    }

    fn parse_assignment(&mut self, ast: &mut AST) -> Result<usize, ParserError> {
        assert_eq!(self.peek(0)?.tt, TokenType::Id);
        assert_eq!(self.peek(1)?.tt, TokenType::Assignment);

        let assid = self.peek(0)?;
        let name = assid.value.clone();
        self.advance();
        self.advance();

        if self.bound.is_bound(&assid.value) {
            return Err(self.parse_error(format!("Identifier already bound: {}", assid.value)));
        }

        let line = self.lexer.line;
        let col = self.lexer.col;

        self.bind(assid.value.clone());
        let exp = match self.peek(0)?.tt {
            TokenType::Lambda => {
                self.advance();
                self.parse_abstraction(ast)?
            }
            _ => self.parse_expression(ast)?,
        };

        let id = ast.add_id(assid, line, col);

        // Ignore if type assignment is not found
        // This is for testing purposes, should be changed to enforce type assignment
        let type_assignment = match self.get_type_assignment(&name) {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        Ok(ast.add_assignment(id, exp, line, col, type_assignment))
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
                    TokenType::DoubleColon => self.parse_type_assignment(&mut ast)?,
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
