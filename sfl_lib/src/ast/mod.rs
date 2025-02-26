mod building;
mod output;
mod transform;

use crate::{find_redexes::RCPair, Primitive, Type, TokenType, Token};
use std::collections::HashSet;
use std::iter::zip;
use std::{collections::HashMap, fmt::Debug, vec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ASTNodeType {
    Identifier,
    Literal,
    Pair,
    Application,
    Assignment,
    Abstraction,
    Module,
    Match,
}

#[derive(Clone)]
pub struct AST {
    vec: Vec<ASTNode>,
    pub root: usize,
}

#[derive(Clone)]
pub struct ASTNode {
    pub t: ASTNodeType,
    info: Option<Token>,
    children: Vec<usize>,
    pub line: usize,
    pub col: usize,
    pub type_assignment: Option<Type>,
    pub wait_for_args: bool,
    pub fancy_assign_abst_syntax: bool,
    pub dollar_app: bool,
    pub is_silent: bool,
}

impl Debug for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = format!("{:?} ", self.t);
        if let Some(tk) = &self.info {
            s.push_str(&format!("{:?} ", tk.value));
        }

        write!(f, "{}", s)
    }
}

impl ASTNode {
    pub fn get_lit_type(&self) -> Type {
        match &self.t {
            ASTNodeType::Literal => match &self.info {
                Some(tk) => match tk.tt {
                    TokenType::IntLit => Type::Primitive(Primitive::Int64),
                    TokenType::FloatLit => Type::Primitive(Primitive::Float64),
                    TokenType::BoolLit => Type::Primitive(Primitive::Bool),
                    TokenType::StringLit => unimplemented!("String literal type"),
                    _ => panic!("Literal node with bad token"),
                },
                None => panic!("Literal node with no token"),
            },
            _ => panic!("get_lit_type called on non-literal node"),
        }
    }

    /// Get the string value of the identifier or literal
    #[inline(always)]
    pub fn get_value(&self) -> String {
        assert!(self.t == ASTNodeType::Identifier || self.t == ASTNodeType::Literal);
        match &self.info {
            Some(tk) => tk.value.clone(),
            None => panic!("Cannot get value of node {:?}", self),
        }
    }

    fn new_lit(tk: Token, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Literal,
            info: Some(tk),
            children: vec![],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
            is_silent: false,
        }
    }

    fn new_id(tk: Token, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Identifier,
            info: Some(tk),
            children: vec![],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
            is_silent: false,
        }
    }

    fn new_pair(a: usize, b: usize, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Pair,
            info: None,
            children: vec![a, b],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
            is_silent: false,
        }
    }

    fn new_app(f: usize, x: usize, line: usize, col: usize, dollar: bool) -> Self {
        ASTNode {
            t: ASTNodeType::Application,
            info: None,
            children: vec![f, x],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: dollar,
            is_silent: false,
        }
    }

    fn new_abstraction(id: usize, exp: usize, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Abstraction,
            info: None,
            children: vec![id, exp],
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
            is_silent: false,
        }
    }

    fn new_assignment(
        id: usize,
        exp: usize,
        line: usize,
        col: usize,
        t: Option<Type>,
        is_silent: bool,
    ) -> Self {
        ASTNode {
            t: ASTNodeType::Assignment,
            info: None,
            children: vec![id, exp],
            line,
            col,
            type_assignment: t,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
            is_silent,
        }
    }

    fn new_module(assigns: Vec<usize>, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Module,
            info: None,
            children: assigns,
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
            is_silent: false,
        }
    }

    fn new_match(cases: Vec<usize>, line: usize, col: usize) -> Self {
        ASTNode {
            t: ASTNodeType::Match,
            info: None,
            children: cases,
            line,
            col,
            type_assignment: None,
            wait_for_args: false,
            fancy_assign_abst_syntax: false,
            dollar_app: false,
            is_silent: false,
        }
    }

    fn wait_for_args(&mut self) {
        self.wait_for_args = true;
    }
}

impl AST {
    pub fn rc_to_str(&self, rc: &RCPair) -> String {
        self.to_string_sugar(rc.0, false) + " -> " + &rc.1.to_string_sugar(rc.1.root, false)
    }

    pub fn print_vec_string(&self) -> String {
        let mut s = String::new();
        for n in &self.vec {
            s.push_str(&format!("{:?}\n", n));
        }
        s
    }

    #[inline(always)]
    pub fn get(&self, i: usize) -> &ASTNode {
        &self.vec[i]
    }

    pub fn get_first(&self, p: usize) -> usize {
        assert_eq!(self.get(p).t, ASTNodeType::Pair);
        self.get(p).children[0]
    }

    pub fn get_second(&self, p: usize) -> usize {
        assert_eq!(self.get(p).t, ASTNodeType::Pair);
        self.get(p).children[1]
    }

    pub fn get_abstr_var(&self, abst: usize) -> usize {
        assert_eq!(self.vec[abst].t, ASTNodeType::Abstraction);
        self.vec[abst].children[0]
    }

    pub fn get_abstr_expr(&self, abst: usize) -> usize {
        assert_eq!(self.vec[abst].t, ASTNodeType::Abstraction);
        self.vec[abst].children[1]
    }

    pub fn get_func(&self, app: usize) -> usize {
        assert_eq!(self.vec[app].t, ASTNodeType::Application);
        self.vec[app].children[0]
    }

    pub fn get_arg(&self, app: usize) -> usize {
        assert_eq!(self.vec[app].t, ASTNodeType::Application);
        self.vec[app].children[1]
    }

    pub fn get_assign_exp(&self, assign: usize) -> usize {
        assert_eq!(self.vec[assign].t, ASTNodeType::Assignment);
        self.vec[assign].children[1]
    }

    pub fn get_assignee(&self, assign: usize) -> String {
        assert_eq!(self.vec[assign].t, ASTNodeType::Assignment);
        self.get(self.vec[assign].children[0]).get_value().clone()
    }

    pub fn get_assignee_names(&self, module: usize) -> Vec<String> {
        let mut names = Vec::new();
        let assigns = &self.vec[module].children;
        names.reserve_exact(assigns.len());
        for a in assigns {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            names.push(id.get_value());
        }

        names
    }

    pub fn get_main(&self, module: usize) -> Option<usize> {
        self.get_assign_to(module, "main".to_string())
    }

    // Get assignment within module
    pub fn get_assign_to(&self, module: usize, name: String) -> Option<usize> {
        assert_eq!(self.vec[module].t, ASTNodeType::Module);

        let assigns = &self.vec[module].children;
        for a in assigns {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            if id.get_value() == name {
                return Some(*a);
            }
        }

        None
    }

    pub fn get_assigns_map(&self, module: usize) -> HashMap<String, usize> {
        assert_eq!(self.vec[module].t, ASTNodeType::Module);
        let mut assigns = HashMap::new();

        for a in &self.vec[module].children {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            assigns.insert(id.get_value(), *a);
        }

        assigns
    }

    pub fn get_match_unpack_pattern(&self, match_: usize) -> usize {
        assert_eq!(self.vec[match_].t, ASTNodeType::Match);
        assert!(self.vec[match_].children.len() > 1);
        self.vec[match_].children[0]
    }

    /// returns patterns to expressions
    pub fn get_match_cases(&self, match_: usize) -> Vec<(usize, usize)> {
        assert_eq!(self.vec[match_].t, ASTNodeType::Match);
        let new_vec = self.vec[match_].children.clone()[1..].to_vec();
        match new_vec.len() % 2 {
            0 => {
                let mut cases = vec![];
                for i in 0..new_vec.len() / 2 {
                    cases.push((new_vec[i * 2], new_vec[i * 2 + 1]));
                }
                cases
            }
            _ => panic!("Match cases must be in pairs"),
        }
    }

    pub fn expr_eq(&self, n1: usize, n2: usize) -> bool {
        match (&self.get(n1).t, &self.get(n2).t) {
            (ASTNodeType::Identifier, ASTNodeType::Identifier)
            | (ASTNodeType::Literal, ASTNodeType::Literal) => {
                self.get(n1).get_value() == self.get(n2).get_value()
            }
            (ASTNodeType::Application, ASTNodeType::Application) => {
                let f1 = self.get_func(n1);
                let f2 = self.get_func(n2);
                let x1 = self.get_arg(n1);
                let x2 = self.get_arg(n2);

                self.expr_eq(f1, f2) && self.expr_eq(x1, x2)
            }
            (ASTNodeType::Abstraction, ASTNodeType::Abstraction) => {
                let v1 = self.get_abstr_var(n1);
                let v2 = self.get_abstr_var(n2);
                let x1 = self.get_abstr_expr(n1);
                let x2 = self.get_abstr_expr(n2);

                self.expr_eq(v1, v2) && self.expr_eq(x1, x2)
            }
            (ASTNodeType::Pair, ASTNodeType::Pair) => {
                let x1 = self.get_first(n1);
                let y1 = self.get_second(n1);
                let x2 = self.get_first(n2);
                let y2 = self.get_second(n2);

                self.expr_eq(x1, x2) && self.expr_eq(y1, y2)
            }
            (ASTNodeType::Match, ASTNodeType::Match) => {
                for (c1, c2) in zip(self.get(n1).children.clone(), self.get(n2).children.clone()) {
                    if !self.expr_eq(c1, c2) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

impl Debug for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_sugar(self.root, false))
    }
}
