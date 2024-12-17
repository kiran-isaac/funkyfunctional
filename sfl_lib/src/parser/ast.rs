use std::{
    collections::HashSet, fmt::{write, Debug}, vec
};

use super::token::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ASTNodeType {
    Identifier,
    Literal,

    Application,
    Assignment,

    Module,
}

pub struct AST {
    vec: Vec<ASTNode>,
}

#[derive(Clone)]
pub struct ASTNode {
    // Ids needed for identifying redexes
    id: usize,

    pub t: ASTNodeType,
    info: Option<Token>,
    children: Vec<usize>,
}

pub enum Type {
    Int,
    Float,
    String,
    Char,
    Bool,
    Unit,
}

impl ASTNode {
    pub fn get_lit_type(&self) -> Type {
        match &self.t {
            ASTNodeType::Literal => match &self.info {
                Some(tk) => match tk.tt {
                    TokenType::IntLit => Type::Int,
                    TokenType::FloatLit => Type::Float,
                    TokenType::StringLit => Type::String,
                    TokenType::CharLit => Type::Char,
                    _ => unreachable!("Literal node with bad token"),
                },
                None => unreachable!("Literal node with no token"),
            },
            _ => unreachable!("get_lit_type called on non-literal node"),
        }
    }

    // Get the string value of the identifier or literal
    pub fn get_value(&self) -> String {
        assert!(self.t == ASTNodeType::Identifier || self.t == ASTNodeType::Literal);
        match &self.info {
            Some(tk) => tk.value.clone(),
            None => unreachable!(),
        }
    }
}

impl AST {
    pub fn new() -> Self {
        Self {
            vec : vec![]
        }
    }

    fn add(&mut self, n : ASTNode) -> usize {
        self.vec.push(n);
        self.vec.len() - 1
    }

    pub fn add_id(&mut self, tk: Token) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Identifier,
            info: Some(tk),
            children: vec![],
            id: self.vec.len(),
        })
    }

    pub fn add_lit(&mut self, tk : Token) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Literal,
            info: Some(tk),
            children: vec![],
            id: self.vec.len(),
        })
    }

    pub fn add_app(&mut self, f : usize, x : usize) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Application,
            info: None,
            children: vec![f, x],
            id: self.vec.len(),
        })
    }

    pub fn new_assignment(&mut self, id : usize, exp : usize) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Assignment,
            info: None,
            children: vec![id, exp],
            id: self.vec.len(),
        })
    }

    pub fn add_module(&mut self, assigns : Vec<usize>) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Module,
            info: None,
            children: assigns,
            id: self.vec.len(),
        })
    }

    pub fn add_to_module(&mut self, module : usize, assign : usize) {
        assert!(self.vec[module].t == ASTNodeType::Module);
        self.vec[module].children.push(assign);
    }

    pub fn get(&self, i : usize) -> &ASTNode {
        &self.vec[i]
    }

    // Get assignment within module
    pub fn get_assign_to(&self, module : usize, name : String) -> Option<usize> {
        assert!(self.vec[module].t == ASTNodeType::Module);

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

    pub fn get_func(&self, app : usize) -> usize {
        assert!(self.vec[app].t == ASTNodeType::Application);
        self.vec[app].children[0]
    }

    pub fn get_arg(&self, app : usize) -> usize {
        assert!(self.vec[app].t == ASTNodeType::Application);
        self.vec[app].children[1]
    }

    pub fn get_exp(&self, assign : usize) -> usize {
        assert!(self.vec[assign].t == ASTNodeType::Assignment);
        self.vec[assign].children[1]
    }

    pub fn get_assignee(&self, assign : usize) -> String {
        assert!(self.vec[assign].t == ASTNodeType::Assignment);
        self.get(self.vec[assign].children[0]).get_value().clone()
    }

    fn to_string_indent(&self, node : usize, indent : usize) -> String {
        let n = self.get(node);
        let ind = " ".repeat(indent);
        match n.t {
            ASTNodeType::Identifier => {
                format!("{}Identifier: {}", ind, n.get_value())
            }
            ASTNodeType::Literal => {
                format!("{}Literal: {}", ind, n.get_value())
            }
            ASTNodeType::Application => {
                let left = self.to_string_indent(self.get_func(node), indent + 2);
                let right = self.to_string_indent(self.get_arg(node), indent + 2);
                format!("{}Application\n{}\n{}", ind, left, right)
            }
            ASTNodeType::Assignment => {
                let id = self.get(self.get(node).children[0]);
                let exp = self.to_string_indent(self.get_exp(node), indent + 2);
                format!("{}Assignment: {}\n{}", ind, id.get_value(), exp)
            }
            ASTNodeType::Module => {
                let mut s = format!("{}Module\n", ind);
                for c in &n.children {
                    s.push_str(&self.to_string_indent(*c, indent + 2));
                }
                s
            }
        }
    }

    pub fn to_string(&self, node : usize) -> String {
        self.to_string_indent(node, 0)
    }
}