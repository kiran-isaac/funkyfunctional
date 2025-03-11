mod building;
mod node;
mod output;
mod transform;

use crate::{find_redexes::RCPair, Token, Type};
pub use node::*;
use std::collections::HashSet;
use std::iter::zip;
use std::{collections::HashMap, fmt::Debug, vec};

#[derive(Clone)]
pub struct AST {
    vec: Vec<ASTNode>,
    pub root: usize,
}

impl AST {
    pub fn new() -> Self {
        Self {
            vec: vec![],
            root: 0,
        }
    }

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
