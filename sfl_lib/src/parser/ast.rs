use std::{
    collections::{HashMap, HashSet},
    fmt::{write, Debug},
    vec,
};

use crate::{types::TypeError, Primitive, Type};

use super::token::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ASTNodeType {
    Identifier,
    Literal,

    Application,
    Assignment,

    Abstraction,

    Module,
}

pub struct AST {
    vec: Vec<ASTNode>,
    pub root: usize,
}

#[derive(Clone)]
pub struct ASTNode {
    pub t: ASTNodeType,
    info: Option<Token>,
    children: Vec<usize>,
    line : usize,
    col : usize
}

impl ASTNode {
    pub fn get_lit_type(&self) -> Type {
        match &self.t {
            ASTNodeType::Literal => match &self.info {
                Some(tk) => match tk.tt {
                    TokenType::IntLit => Type::Primitive(Primitive::Int64),
                    TokenType::FloatLit => Type::Primitive(Primitive::Float64),
                    TokenType::CharLit => Type::Primitive(Primitive::Char),
                    TokenType::BoolLit => Type::Primitive(Primitive::Bool),
                    TokenType::StringLit => unimplemented!("String literal type"),
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
            vec: vec![],
            root: 0,
        }
    }

    fn add(&mut self, n: ASTNode) -> usize {
        self.vec.push(n);
        self.vec.len() - 1
    }

    pub fn replace(&mut self, old: usize, new: usize) {
        // Replace references to the old node with the new node
        self.replace_references_to_node(old, new);
    }

    fn get_all_children_recurse(&self, node : usize) -> Vec<usize> {
        let mut children = vec![];
        for c in &self.get(node).children {
            children.push(*c);
            children.append(&mut self.get_all_children_recurse(*c));
        }
        children
    }

    fn remove(&mut self, node: usize) {
        let children = self.get(node).children.clone();
        for c in children {
            self.remove(c);
        }

        self.assert_no_references(node);
        self.vec.remove(node);
    }

    fn assert_no_references(&self, node: usize) {
        for n in &self.vec {
            for c in &n.children {
                assert!(*c != node);
            }
        }
    }

    fn replace_references_to_node(&mut self, old: usize, new: usize) {
        if self.root == old {
            self.root = new;
        }
        for n in &mut self.vec {
            for c in &mut n.children {
                if *c == old {
                    *c = new;
                }
            }
        }
    }

    // Add a node from another ast to this ast with its children
    pub fn append(&mut self, other: &AST, node: usize) -> usize {
        let n = other.get(node);
        match n.t {
            ASTNodeType::Identifier => self.add_id(n.info.clone().unwrap(), n.line, n.col),
            ASTNodeType::Literal => self.add_lit(n.info.clone().unwrap(), n.line, n.col),
            ASTNodeType::Application => {
                let f = self.append(other, other.get_func(node));
                let x = self.append(other, other.get_arg(node));
                self.add_app(f, x, n.line, n.col)
            }
            ASTNodeType::Assignment => {
                let id = self.append(other, n.children[0]);
                let exp = self.append(other, other.get_exp(node));
                self.new_assignment(id, exp, n.line, n.col)
            }
            ASTNodeType::Module => {
                let mut assigns = vec![];
                for a in n.children.clone() {
                    assigns.push(self.append(other, a));
                }
                self.add_module(assigns, n.line, n.col)
            }
            _ => unimplemented!("append for {:?}", n.t),
        }
    }

    pub fn add_id(&mut self, tk: Token, line : usize, col : usize) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Identifier,
            info: Some(tk),
            children: vec![],
            line,
            col
        })
    }

    pub fn add_lit(&mut self, tk: Token, line : usize, col : usize) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Literal,
            info: Some(tk),
            children: vec![],
            line,
            col
        })
    }

    pub fn add_app(&mut self, f: usize, x: usize, line : usize, col : usize) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Application,
            info: None,
            children: vec![f, x],
            line,
            col
        })
    }

    pub fn add_abstraction(&mut self, id: usize, exp: usize, line : usize, col : usize) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Abstraction,
            info: None,
            children: vec![id, exp],
            line,
            col
        })
    }

    pub fn new_assignment(&mut self, id: usize, exp: usize, line : usize, col : usize) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Assignment,
            info: None,
            children: vec![id, exp],
            line,
            col
        })
    }

    pub fn add_module(&mut self, assigns: Vec<usize>, line : usize, col : usize) -> usize {
        self.add(ASTNode {
            t: ASTNodeType::Module,
            info: None,
            children: assigns,
            line,
            col
        })
    }

    pub fn add_to_module(&mut self, module: usize, assign: usize) {
        assert!(self.vec[module].t == ASTNodeType::Module);
        self.vec[module].children.push(assign);
    }

    pub fn get(&self, i: usize) -> &ASTNode {
        &self.vec[i]
    }

    pub fn get_func(&self, app: usize) -> usize {
        assert!(self.vec[app].t == ASTNodeType::Application);
        self.vec[app].children[0]
    }

    pub fn get_arg(&self, app: usize) -> usize {
        assert!(self.vec[app].t == ASTNodeType::Application);
        self.vec[app].children[1]
    }

    pub fn get_exp(&self, assign: usize) -> usize {
        assert!(self.vec[assign].t == ASTNodeType::Assignment);
        self.vec[assign].children[1]
    }

    pub fn get_assignee(&self, assign: usize) -> String {
        assert!(self.vec[assign].t == ASTNodeType::Assignment);
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

    pub fn get_assigns_map(&self, module: usize) -> HashMap<String, usize> {
        assert!(self.vec[module].t == ASTNodeType::Module);
        let mut assigns  = HashMap::new();

        for a in &self.vec[module].children {
            let assign = self.get(*a);
            let id = self.get(assign.children[0]);
            assigns.insert(id.get_value(), *a);
        }

        assigns
    }

    fn to_string_indent(&self, node: usize, indent: usize) -> String {
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
            ASTNodeType::Abstraction => {
                let id = self.get(n.children[0]);
                let exp = self.to_string_indent(n.children[1], indent + 2);
                format!("{}Abstraction: {}\n{}", ind, id.get_value(), exp)
            }
        }
    }

    pub fn to_string(&self, node: usize) -> String {
        let n = self.get(node);
        match n.t {
            ASTNodeType::Identifier => {
                format!("{}", n.get_value())
            }
            ASTNodeType::Literal => {
                format!("{}", n.get_value())
            }
            ASTNodeType::Application => {
                let func = self.get_func(node);
                let arg = self.get_arg(node);

                let func_str = self.to_string(func);

                // If the func is an abstraction, wrap it in parens
                let func_str = match self.get(func).t {
                    ASTNodeType::Abstraction => format!("({})", func_str),
                    _ => func_str,
                };
                
                let arg_str = self.to_string(arg);
                // If the argument is an application, wrap it in parens
                let arg_str = match self.get(arg).t {
                    ASTNodeType::Application | ASTNodeType::Abstraction => format!("({})", arg_str),
                    _ => arg_str,
                };

                format!("{} {}", func_str, arg_str)
            }
            ASTNodeType::Assignment => {
                let id = self.get(self.get(node).children[0]);
                let exp = self.to_string(self.get_exp(node));
                format!("{} = {}", id.get_value(), exp)
            }
            ASTNodeType::Module => {
                let mut s = String::new();
                for c in &n.children {
                    s.push_str(&self.to_string(*c));
                    s.push_str("\n");
                }

                s.trim().to_string()
            }
            ASTNodeType::Abstraction => {
                let id = self.get(n.children[0]);
                let exp = self.to_string(n.children[1]);
                format!("\\{} . {}", id.get_value(), exp)
            }
        }
    }

    pub fn display_string(&self, node: usize) -> String {
        self.to_string_indent(node, 0)
    }

    fn type_error(&self, e: String, node: usize) -> TypeError {
        let n = self.get(node);
        TypeError {
            e,
            line: n.line,
            col: n.col,
        }
    }

    pub fn get_type(&self, node: usize) -> Result<Type, TypeError> {
        match self.get(node).t {
            ASTNodeType::Literal => Ok(self.get(node).get_lit_type()),
            _ => Err(self.type_error("Smoingus".to_string(), node))
        }
    }
}

impl Debug for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_string(0))
    }
}
