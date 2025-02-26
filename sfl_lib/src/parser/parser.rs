use super::lexer::{Lexer, LexerError};
use super::token::*;
use crate::{AST, ASTNodeType, KnownTypeLabelTable, Type, PRELUDE};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, prelude::*};

pub struct Parser {
    t_queue: VecDeque<Token>,
    lexer: Lexer,
    type_assignment_map: HashMap<String, Type>,
    bound: HashSet<String>,
}

pub struct ParserError {
    pub e: String,
    line: usize,
    col: usize,
}

pub struct TypeMap {
    pub types: HashMap<String, Type>,
}

impl TypeMap {
    pub fn new() -> Self {
        let mut type_decls = HashMap::new();
        type_decls.insert("Int".to_string(), Type::int64());
        type_decls.insert("Float".to_string(), Type::float64());
        type_decls.insert("Bool".to_string(), Type::bool());
        Self { types: type_decls }
    }
}

pub struct ParseResult {
    pub ast: AST,
    pub lt: KnownTypeLabelTable,
    pub tm: TypeMap,
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
            bound: HashSet::new(),
            type_assignment_map: HashMap::new(),
        })
    }

    pub fn from_string(str: String) -> Self {
        Self {
            t_queue: VecDeque::new(),
            lexer: Lexer::new(str, None),
            bound: KnownTypeLabelTable::get_starting_bindings_map()
                .iter()
                .cloned()
                .collect(),
            type_assignment_map: HashMap::new(),
        }
    }

    pub fn add_bindings_from(&mut self, other: &Parser) {
        self.bound.extend(other.bound.clone());
    }

    pub fn bind(&mut self, name: String) {
        self.bound.insert(name);
    }

    pub fn bind_node(&mut self, ast: &mut AST, node: usize) -> Result<(), ParserError> {
        let n = ast.get(node);
        match n.t {
            ASTNodeType::Identifier => {
                let str = n.get_value().clone();
                if self.bound.contains(str.as_str()) {
                    return Err(self.parse_error(format!(
                        "Variable {} is already bound, and cannot be rebound for abstraction",
                        str
                    )));
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
                let str = n.get_value();
                if str != "_" {
                    self.unbind(&str);
                }
            }
            ASTNodeType::Pair => {
                self.unbind_node(ast, ast.get_first(node));
                self.unbind_node(ast, ast.get_second(node));
            }
            _ => panic!("cant bind node"),
        }
    }

    pub fn unbind(&mut self, name: &String) {
        self.bound.remove(name);
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
    fn parse_abstraction(
        &mut self,
        ast: &mut AST,
        is_assign: bool,
        type_table: &HashMap<String, Type>,
    ) -> Result<(usize, Vec<usize>), ParserError> {
        let mut args = vec![];

        loop {
            let t = self.peek(0)?;
            match (t.tt, is_assign) {
                (TokenType::Id | TokenType::LParen, _) => {
                    args.push(self.parse_abstr_var(ast, type_table)?);
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

        let mut expr = self.parse_expression(ast, type_table)?;

        let mut absts_vec = vec![];
        for &&arg in &args.iter().rev().collect::<Vec<&usize>>() {
            expr = ast.add_abstraction(arg, expr, self.lexer.line, self.lexer.col);
            absts_vec.push(expr);
            self.unbind_node(ast, arg);
        }
        Ok((expr, absts_vec))
    }

    fn parse_abstr_var(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let left = self.parse_abstr_var_primary(ast, type_table)?;
        match self.peek(0)?.tt {
            TokenType::Comma => {
                self.advance();
                let right = self.parse_abstr_var(ast, type_table)?;
                Ok(ast.add_pair(left, right, self.lexer.line, self.lexer.col))
            }
            TokenType::DoubleColon => {
                self.advance();
                let type_ = self.parse_type_expression(type_table, None)?;
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

    fn parse_abstr_var_primary(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let t = self.consume()?;
        match t.tt {
            TokenType::Id => Ok(ast.add_id(t, self.lexer.line, self.lexer.col)),
            TokenType::LParen => self.parse_abstr_var(ast, type_table),
            _ => Err(self.parse_error("Expected identifier (or '(') after lambda".to_string())),
        }
    }

    fn parse_expression(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let mut left = self.parse_expr_primary(ast, type_table)?;

        #[cfg(debug_assertions)]
        let _t_queue = format!("{:?}", self.t_queue);
        loop {
            #[cfg(debug_assertions)]
            let _left_str = format!("{:?}", ast.to_string_sugar(left, false));

            let line = self.lexer.line;
            let col = self.lexer.col;
            let tk = self.peek(0)?;
            match &tk.tt {
                // If paren, apply to paren
                TokenType::LParen => {
                    self.advance();
                    let right = self.parse_expression(ast, type_table)?;
                    match self.peek(0)?.tt {
                        TokenType::RParen => {
                            self.advance();
                        }
                        _ => {
                            return Err(self.parse_error(format!(
                                "Expected closing parenthesis, got \"{:?}\"",
                                tk
                            )))
                        }
                    }
                    left = ast.add_app(left, right, line, col, false);
                }

                TokenType::Dollar => {
                    self.advance();
                    let right = self.parse_expression(ast, type_table)?;
                    match self.peek(0)?.tt {
                        TokenType::RParen => {
                            return Err(self.parse_error(format!("Unexpected closing parenthesis")))
                        }
                        _ => {}
                    }
                    left = ast.add_app(left, right, line, col, true);
                }

                TokenType::RParen
                | TokenType::EOF
                | TokenType::Newline
                | TokenType::DoubleColon
                | TokenType::LBrace
                | TokenType::Then
                | TokenType::Else => {
                    return Ok(left);
                }

                TokenType::Comma => {
                    self.advance();
                    let right = self.parse_expression(ast, type_table)?;
                    left = ast.add_pair(left, right, line, col);
                }

                TokenType::Lambda => {
                    self.advance();
                    self.parse_abstraction(ast, false, type_table)?.0;
                }

                TokenType::If => {
                    self.advance();
                    let ite = self.parse_ite(ast, type_table)?;
                    left = ast.add_app(left, ite, line, col, false);
                }

                TokenType::Match => {
                    self.advance();
                    let match_ = self.parse_match(ast, type_table)?;
                    left = ast.add_app(left, match_, line, col, false);
                }

                TokenType::FloatLit
                | TokenType::CharLit
                | TokenType::IntLit
                | TokenType::BoolLit => {
                    let right = self.parse_expr_primary(ast, type_table)?;
                    left = ast.add_app(left, right, line, col, false);
                }

                TokenType::Id | TokenType::UppercaseId => {
                    if self.peek(0)?.is_infix_id() {
                        let id_node = self.parse_expr_primary(ast, type_table)?;
                        let right = self.parse_expression(ast, type_table)?;
                        left = ast.add_app(id_node, left, line, col, false);
                        left = ast.add_app(left, right, line, col, false);
                    } else {
                        let id_node = self.parse_expr_primary(ast, type_table)?;
                        left = ast.add_app(left, id_node, line, col, false);
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
    fn parse_expr_primary(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let line = self.lexer.line;
        let col = self.lexer.col;
        let t = self.consume()?;
        match t.tt {
            TokenType::Id | TokenType::UppercaseId => {
                let id_name = t.value.clone();
                if !self.bound.contains(&id_name) {
                    return Err(self.parse_error(format!("Unbound identifier: {}", id_name)));
                }
                Ok(ast.add_id(t, line, col))
            }
            TokenType::IntLit | TokenType::FloatLit | TokenType::BoolLit | TokenType::CharLit => {
                Ok(ast.add_lit(t, line, col))
            }
            TokenType::If => Ok(self.parse_ite(ast, type_table)?),
            TokenType::Match => Ok(self.parse_match(ast, type_table)?),
            TokenType::Lambda => {
                self.advance();
                Ok(self.parse_abstraction(ast, false, type_table)?.0)
            }
            TokenType::LParen | TokenType::Dollar => {
                let exp = self.parse_expression(ast, type_table)?;
                self.advance();
                Ok(exp)
            }
            _ => Err(self.parse_error(format!("Unexpected Token in primary: {:?}", t))),
        }
    }

    fn parse_pattern<'a>(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
        unpack: bool,
        bound_set: &'a mut HashSet<String>,
    ) -> Result<(usize, &'a mut HashSet<String>), ParserError> {
        let mut left = self
            .parse_pattern_primary(ast, type_table, unpack, bound_set)?
            .0;

        #[cfg(debug_assertions)]
        let _t_queue = format!("{:?}", self.t_queue);
        loop {
            let line = self.lexer.line;
            let col = self.lexer.col;
            match &self.peek(0)?.tt {
                // If paren, apply to paren
                TokenType::LParen => {
                    self.advance();
                    let right = self.parse_pattern(ast, type_table, unpack, bound_set)?.0;
                    self.advance();
                    left = ast.add_app(left, right, line, col, false);
                }

                TokenType::RParen
                | TokenType::RArrow
                | TokenType::LBrace
                | TokenType::EOF
                | TokenType::DoubleColon
                | TokenType::Assignment
                | TokenType::Newline => {
                    return Ok((left, bound_set));
                }

                TokenType::Comma => {
                    self.advance();
                    let right = self.parse_pattern(ast, type_table, unpack, bound_set)?.0;
                    left = ast.add_pair(left, right, line, col);
                }

                TokenType::FloatLit
                | TokenType::CharLit
                | TokenType::IntLit
                | TokenType::BoolLit => {
                    let right = self
                        .parse_pattern_primary(ast, type_table, unpack, bound_set)?
                        .0;
                    left = ast.add_app(left, right, line, col, false);
                }

                TokenType::Id | TokenType::UppercaseId => {
                    // Will throw if lowercase ID is found
                    let id_node = self
                        .parse_pattern_primary(ast, type_table, unpack, bound_set)?
                        .0;
                    left = ast.add_app(left, id_node, line, col, false);
                }

                _ => {
                    let e = format!("Unexpected token in pattern: {:?}", self.peek(0)?);
                    return Err(self.parse_error(e));
                }
            }
        }
    }

    // Parse a primary expression
    fn parse_pattern_primary<'a>(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
        unpack: bool,
        bound_set: &'a mut HashSet<String>,
    ) -> Result<(usize, &'a mut HashSet<String>), ParserError> {
        let line = self.lexer.line;
        let col = self.lexer.col;
        let t = self.consume()?;
        match t.tt {
            TokenType::Id | TokenType::UppercaseId => {
                let id_name = t.value.clone();

                match id_name.chars().next().unwrap() {
                    'A'..='Z' => {
                        if !self.bound.contains(&id_name) {
                            Err(self.parse_error(format!(
                                "Unbound constructor identifier: {}",
                                id_name
                            )))
                        } else {
                            Ok((ast.add_id(t, line, col), bound_set))
                        }
                    }
                    '_' => Ok((ast.add_id(t, line, col), bound_set)),
                    'a'..='z' => {
                        if unpack {
                            if self.bound.contains(&id_name) {
                                return Err(self.parse_error(format!(
                                    "Cannot rebind already bound identifier: {}",
                                    id_name
                                )));
                            } else {
                                bound_set.insert(id_name.clone());
                            }
                        } else if !unpack && !self.bound.contains(&id_name) {
                            return Err(
                                self.parse_error(format!("Unbound Identifier: {}", id_name))
                            );
                        }
                        Ok((ast.add_id(t, line, col), bound_set))
                    }
                    _ => Err(self.parse_error(format!("unexpected char in id: {}", t.value))),
                }
            }
            TokenType::IntLit | TokenType::FloatLit | TokenType::BoolLit | TokenType::CharLit => {
                Ok((ast.add_lit(t, line, col), bound_set))
            }
            TokenType::LParen => {
                let exp = self.parse_pattern(ast, type_table, unpack, bound_set)?.0;
                self.advance();
                Ok((exp, bound_set))
            }

            _ => Err(self.parse_error(format!("Unexpected Token in pattern primary: {:?}", t))),
        }
    }

    fn parse_match(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        // Parse the expression to match on, this does not allow literals
        let match_unpack = self.parse_expression(ast, type_table)?;

        match self.consume()?.tt {
            TokenType::DoubleColon => {
                ast.set_type(match_unpack, self.parse_type_expression(type_table, None)?);
                match self.consume()?.tt {
                    TokenType::LBrace => {}
                    _ => {
                        return Err(self.parse_error(
                            "Expected \"{\" after match type assignment before cases".to_string(),
                        ));
                    }
                };
            }
            TokenType::LBrace => {}
            _ => {
                return Err(self.parse_error(
                    "Expected type assignment of match subject, or lbrace".to_string(),
                ));
            }
        };

        let mut children = vec![match_unpack];

        loop {
            let t = self.peek(0)?;

            match self.peek(0)?.tt {
                TokenType::RBrace => {
                    self.advance();
                    break;
                }
                TokenType::Newline => {
                    self.advance();
                }
                TokenType::Bar => {
                    let mut bound_set = HashSet::new();
                    let bar = self.consume()?;
                    match bar.tt {
                        TokenType::Bar => {}
                        _ => {
                            return Err(
                                self.parse_error("Expected \"|\" before case pattern".to_string())
                            );
                        }
                    };

                    let case = self.parse_pattern(ast, type_table, true, &mut bound_set)?.0;

                    let arrow = self.consume()?;
                    match arrow.tt {
                        TokenType::RArrow => {}
                        _ => {
                            return Err(
                                self.parse_error("Expected \"->\" after case pattern".to_string())
                            );
                        }
                    };

                    for item in bound_set.iter() {
                        self.bind(item.clone())
                    }
                    let expr = self.parse_expression(ast, type_table)?;
                    for item in bound_set.iter() {
                        self.unbind(item)
                    }
                    children.push(case);
                    children.push(expr);
                }
                _ => {
                    return Err(self.parse_error(format!(
                        "Unexpected Token in match: expected \"|\", got: {:?}",
                        t
                    )))
                }
            }
        }

        Ok(ast.add_match(children, self.lexer.line, self.lexer.col))
    }

    fn parse_ite(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let if_id_node = ast.add_id(
            Token {
                tt: TokenType::If,
                value: "if".to_string(),
            },
            self.lexer.line,
            self.lexer.col - 2,
        );

        let condition = self.parse_expression(ast, type_table)?;

        let app1 = ast.add_app(
            if_id_node,
            condition,
            self.lexer.line,
            self.lexer.col,
            false,
        );

        let then_tk = self.consume()?;
        if then_tk.tt != TokenType::Then {
            return Err(self.parse_error(format!("Expected \"then\", got: {}", then_tk.value)));
        }

        let then_exp = self.parse_expression(ast, type_table)?;
        let app2 = ast.add_app(app1, then_exp, self.lexer.line, self.lexer.col, false);

        let else_tk = self.consume()?;
        if else_tk.tt != TokenType::Else {
            return Err(self.parse_error(format!("Expected \"else\", got: {}", then_tk.value)));
        }

        let else_exp = self.parse_expression(ast, type_table)?;
        let app3 = ast.add_app(app2, else_exp, self.lexer.line, self.lexer.col, false);

        Ok(app3)
    }

    fn parse_type_expression(
        &mut self,
        type_table: &HashMap<String, Type>,
        bound_type_vars: Option<&HashSet<String>>,
    ) -> Result<Type, ParserError> {
        let mut left = self.parse_type_expression_primary(type_table, bound_type_vars)?;

        loop {
            let next = self.peek(0)?;

            match next.tt {
                TokenType::RArrow => {
                    self.advance();
                    let right = self.parse_type_expression(type_table, bound_type_vars)?;

                    left = Type::Function(Box::new(left), Box::new(right));
                }

                TokenType::Comma => {
                    self.advance();
                    left = Type::pr(
                        left,
                        self.parse_type_expression(type_table, bound_type_vars)?,
                    );
                }

                TokenType::UppercaseId | TokenType::Id | TokenType::LParen => {
                    match next.tt {
                        // If this is in an abstraction, and the next token is a double colon, then we're done because
                        // the next ID is another abst variable
                        TokenType::Id => {
                            if self.peek(1)?.tt == TokenType::DoubleColon {
                                return Ok(left);
                            }
                        }
                        _ => {}
                    }

                    let t2 = self.parse_type_expression_primary(type_table, bound_type_vars)?;
                    left = match left.type_app(&t2) {
                        Ok(t) => t,
                        Err(e) => return Err(self.parse_error(e.to_string())),
                    }
                }

                TokenType::RParen
                | TokenType::Newline
                | TokenType::EOF
                | TokenType::Dot
                | TokenType::LBrace => return Ok(left),

                _ => {
                    return Err(self
                        .parse_error(format!("Unexpected token in type expression: {:?}", next)))
                }
            }
        }
    }

    fn parse_type_expression_primary(
        &mut self,
        type_table: &HashMap<String, Type>,
        bound_type_vars: Option<&HashSet<String>>,
    ) -> Result<Type, ParserError> {
        let t = self.consume()?;

        match t.tt {
            TokenType::Id => {
                if let None = bound_type_vars {
                    return Ok(Type::TypeVariable(t.value));
                }

                let id = t.value;
                if bound_type_vars.unwrap().contains(&id) {
                    Ok(Type::TypeVariable(id))
                } else {
                    Err(self.parse_error(format!("Type variable {} is not bound", id)))
                }
            }
            TokenType::UppercaseId => {
                let id = t.value;
                if let Some(t_match) = type_table.get(&id) {
                    // Ok(Type::Alias(id, Box::new(t_match.clone())))
                    Ok(t_match.clone())
                } else {
                    Err(self.parse_error(format!("Type {} is not defined", id)))
                }
            }
            TokenType::LParen => {
                let inner = self.parse_type_expression(type_table, bound_type_vars)?;
                self.advance();
                Ok(inner)
            }
            _ => Err(self.parse_error(format!(
                "Unexpected token in type expression primary: {:?}",
                t
            ))),
        }
    }

    fn parse_type_annotation(
        &mut self,
        type_table: &HashMap<String, Type>,
    ) -> Result<Type, ParserError> {
        let assigned_type = self.parse_type_expression(type_table, None)?;

        let mut sorted_tvs: Vec<String> = assigned_type
            .get_tvs_set()
            .iter()
            .map(|tv| tv.clone())
            .collect();

        sorted_tvs.sort();

        let assigned_type = Type::fa(sorted_tvs, assigned_type);

        #[cfg(debug_assertions)]
        let _assigned_type_str = assigned_type.to_string();

        Ok(assigned_type)
    }

    fn parse_type_assignment(
        &mut self,
        type_map: &HashMap<String, Type>,
    ) -> Result<(), ParserError> {
        let name = self.peek(0)?.value.clone();
        if self.type_assignment_map.contains_key(&name) {
            return Err(self.parse_error(format!("Type already assigned: {}", name)));
        }
        self.advance();
        self.advance();

        let assigned_type = self.parse_type_annotation(type_map)?;

        self.type_assignment_map.insert(name, assigned_type);

        Ok(())
    }

    pub fn get_type_assignment(&self, name: &String) -> Result<Type, ParserError> {
        match self.type_assignment_map.get(name) {
            Some(t) => Ok(t.clone()),
            None => Err(self.parse_error(format!("Type not assigned: {}", name))),
        }
    }

    fn parse_assignment(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        let mut ass_tk = self.peek(0)?;
        assert!(
            ass_tk.tt == TokenType::Id
                || ass_tk.tt == TokenType::If
                || ass_tk.tt == TokenType::Silence
        );

        let is_silent = ass_tk.tt == TokenType::Silence;
        if is_silent {
            self.advance();
            ass_tk = self.consume()?;
            if ass_tk.tt != TokenType::If && ass_tk.tt != TokenType::Id {
                return Err(self
                    .parse_error("Expected id after silence operator in assignment".to_string()));
            };
        }

        let name = ass_tk.value.clone();

        if self.bound.contains(name.as_str()) {
            return Err(self.parse_error(format!("Variable already assigned: {}", name)));
        }

        self.bind(name.clone());

        self.advance();

        let t = self.peek(0)?;
        let mut patterns = vec![];
        let mut bound_set = HashSet::new();

        loop {
            match t.tt {
                TokenType::Id | TokenType::UppercaseId | TokenType::LParen => {
                    let (pattern, _) =
                        self.parse_pattern(ast, type_table, true, &mut bound_set)?;

                    patterns.push(pattern);
                }
                _ => break
            }
        }

        let expr = match t.tt {
            TokenType::Assignment => {
                self.advance();
                self.parse_expression(ast, type_table)?
            }

            _ => {
                return Err(self.parse_error(format!("Unexpected token in assignment: {}", t.value)))
            }
        };

        let id = ast.add_id(ass_tk, self.lexer.line, self.lexer.col);

        // Ignore if type assignment is not found, so the typechecker will have to infer
        let type_assignment = match self.get_type_assignment(&name) {
            Ok(t) => Some(t),
            Err(_) => None,
        };

        Ok(ast.add_assignment(
            id,
            expr,
            self.lexer.line,
            self.lexer.col,
            type_assignment,
            is_silent,
        ))
    }

    /// Takes type table, returns the name of the data and also the type constructors
    fn parse_type_alias_decl(
        &mut self,
        type_table: &HashMap<String, Type>,
    ) -> Result<(String, Type), ParserError> {
        assert_eq!(self.consume()?.tt, TokenType::KWType);

        let t = self.consume()?;
        let name = match t.tt {
            TokenType::UppercaseId => t.value,
            TokenType::Id => {
                return Err(self.parse_error(format!(
                    "Type IDs must begin with a capital letter. Got {}",
                    t.value
                )))
            }
            _ => {
                return Err(self.parse_error(format!(
                    "Expected type ID after type assignment, got {}",
                    t.value
                )))
            }
        };

        let t = self.consume()?;
        match t.tt {
            TokenType::Assignment => {}
            _ => {
                return Err(self.parse_error(format!(
                    "Expected \"=\" after type assignment, got {}",
                    t.value
                )))
            }
        }

        Ok((
            name,
            self.parse_type_expression(type_table, Some(&HashSet::new()))?,
        ))
    }

    fn parse_multiple_constructors(
        &mut self,
        type_table: &HashMap<String, Type>,
        params: &Vec<String>,
        union_type: &Type,
    ) -> Result<HashMap<String, Type>, ParserError> {
        let mut constructors = HashMap::new();
        let bound_type_vars: HashSet<String> = params.iter().cloned().collect();

        loop {
            let t = self.peek(0)?;
            match t.tt {
                TokenType::UppercaseId => {
                    let (constructor_name, constructor_params) =
                        self.parse_constructor(type_table, &bound_type_vars)?;

                    let mut constructor_type = union_type.clone();
                    for param in constructor_params.iter().rev() {
                        constructor_type = Type::f(param.clone(), constructor_type);
                    }

                    // forall-ify it
                    constructor_type = Type::fa(params.clone(), constructor_type);

                    #[cfg(debug_assertions)]
                    let _constructor_type_str = constructor_type.to_string();

                    constructors.insert(constructor_name, constructor_type);
                }
                TokenType::Bar => {
                    self.advance();
                }
                TokenType::Newline | TokenType::EOF => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(self.parse_error(format!(
                        "Unexpected token during data declaration: {}",
                        t.value
                    )))
                }
            }
        }

        Ok(constructors)
    }

    fn parse_constructor(
        &mut self,
        type_table: &HashMap<String, Type>,
        bound_type_vars: &HashSet<String>,
    ) -> Result<(String, Vec<Type>), ParserError> {
        let t = self.consume()?;
        if t.tt != TokenType::UppercaseId {
            return Err(self.parse_error(format!("Expected varient name, got {}", t.value)));
        }
        let constructor_name = t.value;

        let mut constructor_params = vec![];
        loop {
            let t = self.peek(0)?;
            match t.tt {
                TokenType::Id => {
                    self.advance();
                    if bound_type_vars.contains(&t.value) {
                        constructor_params.push(Type::TypeVariable(t.value));
                    } else {
                        return Err(
                            self.parse_error(format!("Unbound type parameter: {}", &t.value))
                        );
                    }
                }
                TokenType::UppercaseId => {
                    self.advance();
                    if let Some(type_) = type_table.get(&t.value) {
                        constructor_params.push(type_.clone());
                    } else {
                        return Err(
                            self.parse_error(format!("Unbound type parameter: {}", &t.value))
                        );
                    }
                }
                TokenType::LParen => {
                    self.advance();

                    let type_ = self.parse_type_expression(type_table, Some(bound_type_vars))?;
                    constructor_params.push(type_);
                    assert_eq!(self.consume()?.tt, TokenType::RParen);
                }

                _ => return Ok((constructor_name, constructor_params)),
            }
        }
    }

    fn parse_data_decl(
        &mut self,
        type_table: &mut HashMap<String, Type>,
    ) -> Result<HashMap<String, Type>, ParserError> {
        assert_eq!(self.consume()?.tt, TokenType::KWData);

        let t = self.consume()?;
        let name = match t.tt {
            TokenType::UppercaseId => t.value,
            TokenType::Id => {
                return Err(self.parse_error(format!(
                    "Type IDs must begin with a capital letter. Got {}",
                    t.value
                )))
            }
            _ => {
                return Err(self.parse_error(format!(
                    "Expected type ID after data keyword, got {}",
                    t.value
                )))
            }
        };

        // parse the params
        let mut tparams = Vec::new();
        let mut t = self.consume()?;
        while t.tt == TokenType::Id {
            if tparams.contains(&t.value) {
                return Err(self.parse_error(format!("Duplicate data parameter: {}", t.value)));
            }
            tparams.push(t.value);
            t = self.consume()?;
        }

        if t.tt != TokenType::Assignment {
            return Err(self.parse_error(format!(
                "Expected \"=\" after data keyword, got {}",
                t.value
            )));
        }

        let union_type = Type::Union(
            name.clone(),
            tparams.iter().map(|v| Type::tv(v.clone())).collect(),
        );

        if let Some(_) = type_table.get(&name) {
            return Err(self.parse_error(format!("Type {} declared more than once", &name)));
        }

        type_table.insert(name.clone(), Type::fa(tparams.clone(), union_type.clone()));

        let constructors = self.parse_multiple_constructors(type_table, &tparams, &union_type)?;
        Ok(constructors)
    }

    fn init_parser(&mut self, with_prelude: bool) -> (KnownTypeLabelTable, TypeMap, AST) {
        if with_prelude {
            let mut parser = Self::from_string(PRELUDE.to_string());
            let pr = match parser.parse_module(false) {
                Ok(pr) => pr,
                Err(e) => panic!("Failed to parse prelude: {:?}", e),
            };
            for binding in parser.bound {
                self.bind(binding);
            }
            (pr.lt, pr.tm, pr.ast)
        } else {
            let mut ast = AST::new();
            let module = ast.add_module(Vec::new(), self.lexer.line, self.lexer.col);
            ast.root = module;
            (KnownTypeLabelTable::new(), TypeMap::new(), ast)
        }
    }

    pub fn parse_module(&mut self, with_prelude: bool) -> Result<ParseResult, ParserError> {
        let (mut lt, mut tm, mut ast) = self.init_parser(with_prelude);
        let module = ast.root;
        let mut main_found = false;

        'assloop: loop {
            let t = self.peek(0)?;

            match t.tt {
                TokenType::Id | TokenType::If | TokenType::Silence => {
                    let next = self.peek(1)?;
                    match next.tt {
                        TokenType::Assignment
                        | TokenType::Id
                        | TokenType::If
                        | TokenType::LParen => {
                            let assignment = self.parse_assignment(&mut ast, &tm.types)?;
                            let ass_node = ast.get(assignment);
                            let ass_name = ast.get_assignee(assignment);
                            if let Some(ass_type) = &ass_node.type_assignment {
                                lt.add(ass_name.clone(), ass_type.clone(), ass_node.is_silent)
                            }
                            if ass_name == "main" {
                                main_found = true;
                            }
                            ast.add_to_module(module, assignment);
                        }
                        TokenType::DoubleColon => self.parse_type_assignment(&tm.types)?,
                        _ => {
                            return Err(self.parse_error(format!(
                                "Unexpected Token: {:?}. Expected assignment operator: =",
                                next.value
                            )))
                        }
                    }
                }
                TokenType::KWType => {
                    let (decl_name, decl_type) = self.parse_type_alias_decl(&tm.types)?;
                    if let Some(_) = tm.types.get(&decl_name) {
                        return Err(self
                            .parse_error(format!("Type {} declared more than once", &decl_name)));
                    }
                    #[cfg(debug_assertions)]
                    let _decl_type_str = decl_type.to_string();

                    tm.types.insert(
                        decl_name.clone(),
                        Type::Alias(decl_name, Box::new(decl_type)),
                    );
                }
                TokenType::KWData => {
                    let constructors = self.parse_data_decl(&mut tm.types)?;

                    for (constructor_name, constructor_type) in constructors {
                        lt.add(constructor_name.clone(), constructor_type, false);
                        self.bind(constructor_name);
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

        if with_prelude && !main_found {
            return Err(self.parse_error(
                "Assignment to 'main' is missing. This is the programs entry point.".to_string(),
            ));
        }

        Ok(ParseResult { ast, lt, tm })
    }

    #[cfg(test)]
    pub fn parse_tl_expression(&mut self, with_prelude: bool) -> Result<ParseResult, ParserError> {
        let (lt, tm, mut ast) = self.init_parser(with_prelude);
        ast.root = self.parse_expression(&mut ast, &tm.types)?;
        Ok(ParseResult { lt, tm, ast })
    }
}
