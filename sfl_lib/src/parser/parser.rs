use super::ast::AST;
use super::bound::BoundChecker;
use super::lexer::{Lexer, LexerError};
use super::token::*;
use crate::{ASTNodeType, Primitive, Type};
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
    pub e: String,
    line: usize,
    col: usize,
}

impl Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Parser Error at [{}:{}]: {}",
            self.line + 1,
            self.col + 1,
            self.e
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

    pub fn bind_node(&mut self, ast: &mut AST, node: usize) -> Result<(), ParserError> {
        let n = ast.get(node);
        match n.t {
            ASTNodeType::Identifier => {
                let str = n.get_value().clone();
                if self.bound.is_bound(str.as_str()) {
                    return Err(self.parse_error(format!("Variable {} is already bound, and cannot be rebound for abstraction", str)));
                }
                if str != "_" {
                    self.bind(str);
                }
                Ok(())
            }
            ASTNodeType::Pair => {
                self.bind_node(ast, ast.get_first(node))?;
                self.bind_node(ast, ast.get_second(node))?;
                Ok(())
            }
            _ => panic!("cant bind node"),
        }
    }

    pub fn unbind_node(&mut self, ast: &mut AST, node: usize) {
        let n = ast.get(node);
        match n.t {
            ASTNodeType::Identifier => {
                let str = n.get_value().clone();
                if str != "_" {
                    self.unbind(str);
                }
            }
            ASTNodeType::Pair => {
                self.unbind_node(ast, ast.get_first(node));
                self.unbind_node(ast, ast.get_second(node));
            }
            _ => panic!("cant bind node"),
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
    #[inline(always)]
    fn queue_tk(&mut self) -> Result<(), ParserError> {
        let t = self.lexer.get_token()?;
        self.t_queue.push_back(t);

        Ok(())
    }

    // Roll queue forwards
    #[inline(always)]
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

    // Parse potentially multiple abstraction, return the abstr node and all the absts as a vector
    fn parse_abstraction(&mut self, ast: &mut AST, is_assign : bool) -> Result<(usize, Vec<usize>), ParserError> {
        let mut args = vec![];

        loop {
            let t = self.peek(0)?;
            match (t.tt, is_assign) {
                (TokenType::Id | TokenType::LParen, _) => {
                    args.push(self.parse_abstr_var(ast)?);
                }
                (TokenType::Dot, false) => break,
                (TokenType::Assignment, true) => break,
                _ => {
                    return Err(self
                        .parse_error(format!("Unexpected token in lambda argument: {}", t.value)))
                }
            }
        }

        if is_assign {
            assert_eq!(self.consume()?.tt, TokenType::Assignment);
        } else {
            assert_eq!(self.consume()?.tt, TokenType::Dot);
        }

        for arg in &args {
            match self.bind_node(ast, *arg) {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }

        let mut expr = self.parse_expression(ast)?;

        let mut absts_vec = vec![];
        for &&arg in &args.iter().rev().collect::<Vec<&usize>>() {
            expr = ast.add_abstraction(arg, expr, self.lexer.line, self.lexer.col);
            absts_vec.push(expr);
            self.unbind_node(ast, arg);
        }
        Ok((expr, absts_vec))
    }

    fn parse_abstr_var(&mut self, ast: &mut AST) -> Result<usize, ParserError> {
        let left = self.parse_abstr_var_primary(ast)?;
        match self.peek(0)?.tt {
            TokenType::Comma => {
                self.advance();
                let right = self.parse_abstr_var(ast)?;
                Ok(ast.add_pair(left, right, self.lexer.line, self.lexer.col))
            }
            TokenType::DoubleColon => {
                self.advance();
                let type_ = self.parse_type_expression(ast)?;
                ast.set_type(left, type_);
                Ok(left)
            }
            TokenType::RParen => {
                self.advance();
                Ok(left)
            }
            _ => Ok(left),
        }
    }

    fn parse_abstr_var_primary(&mut self, ast: &mut AST) -> Result<usize, ParserError> {
        let t = self.consume()?;
        match t.tt {
            TokenType::Id => Ok(ast.add_id(t, self.lexer.line, self.lexer.col)),
            TokenType::LParen => self.parse_abstr_var(ast),
            _ => Err(self.parse_error("Expected identifier (or '(') after lambda".to_string())),
        }
    }

    fn parse_expression(&mut self, ast: &mut AST) -> Result<usize, ParserError> {
        #[cfg(debug_assertions)]
        let mut left = self.parse_primary(ast)?;

        let _t_queue = format!("{:?}", self.t_queue);
        loop {
            let line = self.lexer.line;
            let col = self.lexer.col;
            match &self.peek(0)?.tt {
                // If paren, apply to paren
                TokenType::LParen => {
                    self.advance();
                    let right = self.parse_expression(ast)?;
                    self.advance();
                    left = ast.add_app(left, right, line, col);
                }
                TokenType::RParen
                | TokenType::EOF
                | TokenType::Newline
                | TokenType::Then
                | TokenType::Else => {
                    return Ok(left);
                }

                TokenType::Comma => {
                    self.advance();
                    let right = self.parse_expression(ast)?;
                    left = ast.add_pair(left, right, line, col);
                }

                TokenType::Lambda => {
                    self.advance();
                    self.parse_abstraction(ast, false)?.0;
                }

                TokenType::If => {
                    self.advance();
                    let ite = self.parse_ite(ast)?;
                    left = ast.add_app(left, ite, line, col);
                }

                TokenType::FloatLit
                | TokenType::CharLit
                | TokenType::IntLit
                | TokenType::BoolLit => {
                    let right = self.parse_primary(ast)?;
                    left = ast.add_app(left, right, line, col);
                }

                TokenType::Id => {
                    if self.peek(0)?.is_infix_id() {
                        let id_node = self.parse_primary(ast)?;
                        let right = self.parse_expression(ast)?;
                        left = ast.add_app(id_node, left, line, col);
                        left = ast.add_app(left, right, line, col);
                    } else {
                        let id_node = self.parse_primary(ast)?;
                        left = ast.add_app(left, id_node, line, col);
                    }
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
            TokenType::If => Ok(self.parse_ite(ast)?),
            // Removed support for lambda except at the top level
            // for now, untill i figure out type inference
            TokenType::Lambda => {
                self.advance();
                Ok(self.parse_abstraction(ast, false)?.0)
            }
            TokenType::LParen => {
                let exp = self.parse_expression(ast)?;
                self.advance();
                Ok(exp)
            }
            _ => Err(self.parse_error(format!("Unexpected Token in primary: {:?}", t))),
        }
    }

    fn parse_ite(&mut self, ast: &mut AST) -> Result<usize, ParserError> {
        let if_id_node = ast.add_id(
            Token {
                tt: TokenType::If,
                value: "if".to_string(),
            },
            self.lexer.line,
            self.lexer.col - 2,
        );

        let condition = self.parse_expression(ast)?;

        let app1 = ast.add_app(if_id_node, condition, self.lexer.line, self.lexer.col);

        let then_tk = self.consume()?;
        assert!(then_tk.tt == TokenType::Then);

        let then_exp = self.parse_expression(ast)?;
        let app2 = ast.add_app(app1, then_exp, self.lexer.line, self.lexer.col);

        let else_tk = self.consume()?;
        assert!(else_tk.tt == TokenType::Else);

        let else_exp = self.parse_expression(ast)?;
        let app3 = ast.add_app(app2, else_exp, self.lexer.line, self.lexer.col);

        Ok(app3)
    }

    fn parse_type_expression(&mut self, ast: &mut AST) -> Result<Type, ParserError> {
        let mut left = self.parse_type_expression_primary(ast)?;

        loop {
            let next = self.peek(0)?;

            match next.tt {
                TokenType::RArrow | TokenType::LParen => {
                    self.advance();
                    let right = self.parse_type_expression(ast)?;
                    left = Type::Function(Box::new(left), Box::new(right));
                }

                TokenType::Comma => {
                    self.advance();
                    left = Type::pr(left, self.parse_type_expression(ast)?);
                }

                TokenType::RParen
                | TokenType::Newline
                | TokenType::EOF
                | TokenType::Id
                | TokenType::Dot => return Ok(left),
                _ => {
                    return Err(self
                        .parse_error(format!("Unexpected token in type expression: {:?}", next)))
                }
            }
        }
    }

    fn parse_forall(&mut self, ast: &mut AST) -> Result<Type, ParserError> {
        let mut vars = vec![];
        while self.peek(0)?.tt == TokenType::Id {
            let v = self.peek(0)?.value;
            if self.peek(0)?.is_infix_id() {
                return Err(self.parse_error(format!(
                    "Invalid identifier in forall type declaration: {}",
                    v
                )));
            }
            self.advance();
            vars.push(v);
        }

        if vars.is_empty() {
            return Err(self.parse_error("Invalid forall declaration, no variables".to_string()));
        }

        let tk = self.consume()?;
        if tk.tt != TokenType::Dot {
            return Err(self.parse_error(format!(
                "Invalid forall declaration, expected dot after variables, got {:?}",
                tk
            )));
        }

        Ok(Type::fa(vars, self.parse_type_expression(ast)?))
    }

    fn parse_type_expression_primary(&mut self, ast: &mut AST) -> Result<Type, ParserError> {
        let t = self.consume()?;

        match t.tt {
            TokenType::TypeId | TokenType::Id => {
                let id = t.value;
                match id.as_str() {
                    "Int" => Ok(Type::Primitive(Primitive::Int64)),
                    "Float" => Ok(Type::Primitive(Primitive::Float64)),
                    "Bool" => Ok(Type::Primitive(Primitive::Bool)),
                    _ => Ok(Type::TypeVariable(id)),
                }
            }
            TokenType::LParen => {
                let inner = self.parse_type_expression(ast)?;
                self.advance();
                Ok(inner)
            }
            TokenType::Forall => self.parse_forall(ast),
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

        let assid = self.peek(0)?;
        let name = assid.value.clone();

        if self.bound.is_bound(name.as_str()) {
            return Err(self.parse_error(format!("Variable already assigned: {}", name)));
        }

        self.bind(name.clone());

        self.advance();

        let t = self.peek(0)?;
        let expr = match t.tt {
            TokenType::Assignment => {
                self.advance();
                self.parse_expression(ast)?
            }
            TokenType::Id | TokenType::LParen => {
                let (expr, abst_vars) = self.parse_abstraction(ast, true)?;
                for var in abst_vars.into_iter().rev() {
                    ast.fancy_assign_abst_syntax(var);
                    ast.wait_for_args(var);
                }
                expr
            }
            _ => return Err(self.parse_error(format!("Unexpected token in assignment: {}", t.value)))
        };

        let id = ast.add_id(assid, self.lexer.line, self.lexer.col);

        // Ignore if type assignment is not found, so the typechecker will have to infer
        let type_assignment = match self.get_type_assignment(&name) {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        Ok(ast.add_assignment(id, expr, self.lexer.line, self.lexer.col, type_assignment))
    }

    pub fn parse_module(&mut self) -> Result<AST, ParserError> {
        // At the top level its just a set of assignments
        let mut ast = AST::new();
        let module = ast.add_module(Vec::new(), self.lexer.line, self.lexer.col);

        'assloop: loop {
            let t = self.peek(0)?;

            match t.tt {
                TokenType::Id => {
                    let next = self.peek(1)?;
                    match next.tt {
                        TokenType::Assignment | TokenType::Id | TokenType::LParen => {
                            let assignment = self.parse_assignment(&mut ast)?;
                            ast.add_to_module(module, assignment);
                        }
                        TokenType::DoubleColon => self.parse_type_assignment(&mut ast)?,
                        _ => {
                            return Err(self.parse_error(format!(
                                "Unexpected Token: {:?}. Expected assignment operator: =",
                                next.value
                            )))
                        }
                    }
                }
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
