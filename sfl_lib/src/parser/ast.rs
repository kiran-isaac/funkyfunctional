use std::{
    fmt::{write, Debug},
    vec,
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

#[derive(Clone)]
pub struct ASTNode {
    // Ids needed for identifying redexes
    id: usize,

    pub t: ASTNodeType,
    info: Option<Token>,
    children: Vec<ASTNode>,
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
    pub fn new_id(tk: Token, n: usize) -> Self {
        Self {
            t: ASTNodeType::Identifier,
            info: Some(tk),
            children: vec![],
            id: n,
        }
    }

    pub fn new_lit(tk: Token, n: usize) -> Self {
        Self {
            t: ASTNodeType::Literal,
            info: Some(tk),
            children: vec![],
            id: n,
        }
    }

    pub fn new_app(f: ASTNode, arg: ASTNode, n: usize) -> Self {
        Self {
            t: ASTNodeType::Application,
            info: None,
            children: vec![f, arg],
            id: n,
        }
    }

    pub fn new_assignment(id: Token, exp: ASTNode, n: usize) -> Self {
        Self {
            t: ASTNodeType::Assignment,
            info: None,
            children: vec![Self::new_id(id, n), exp],
            id: n,
        }
    }

    pub fn new_module(children: Vec<ASTNode>, n: usize) -> Self {
        Self {
            t: ASTNodeType::Module,
            info: None,
            children,
            id: n,
        }
    }

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

    // Get the function of an application node
    pub fn get_func(&self) -> &ASTNode {
        assert!(self.t == ASTNodeType::Application);
        assert!(self.children.len() == 2);
        &self.children[0]
    }

    // Get the argument of an application node
    pub fn get_arg(&self) -> &ASTNode {
        assert!(self.t == ASTNodeType::Application);
        assert!(self.children.len() == 2);
        &self.children[1]
    }

    // Get the expression of an assignment node
    pub fn get_exp(&self) -> &ASTNode {
        assert!(self.t == ASTNodeType::Assignment);
        assert!(self.children.len() == 2);
        &self.children[1]
    }

    pub fn get_assignee(&self) -> &ASTNode {
        assert!(self.t == ASTNodeType::Assignment);
        assert!(self.children.len() == 2);
        &self.children[0]
    }

    pub fn get_assign_to(&self, name: String) -> Result<&ASTNode, ()> {
        assert!(self.t == ASTNodeType::Module);
        for child in &self.children {
            if child.get_assignee().get_value() == name {
                return Ok(child);
            }
        }
        Err(())
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    fn to_string(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut s = format!("{}- ", &indent_str);

        match &self.t {
            ASTNodeType::Identifier => {
                let vname = match &self.info {
                    Some(tk) => &tk.value,
                    None => unreachable!(),
                };

                s.push_str("Id: ");
                s.push_str(&vname);
                s
            }
            ASTNodeType::Literal => {
                let value = match &self.info {
                    Some(tk) => &tk.value,
                    None => unreachable!(),
                };

                s.push_str("Id: ");
                s.push_str(&value);
                s
            }
            ASTNodeType::Application => {
                assert!(self.children.len() == 2);
                let f = &self.children[0];
                let x = &self.children[1];

                s.push_str("Application:\n");
                s.push_str(&f.to_string(indent + 1));
                s.push('\n');
                s.push_str(&x.to_string(indent + 1));
                s
            }
            ASTNodeType::Assignment => {
                assert!(self.children.len() == 2);
                let id = &self.children[0];
                let exp = &self.children[1];

                s.push_str("Assignment:\n");
                s.push_str(&id.to_string(indent + 1));
                s.push('\n');
                s.push_str(&exp.to_string(indent + 1));
                s.push('\n');
                s
            }
            ASTNodeType::Module => {
                s.push_str("Module:\n");
                for ass in &self.children {
                    s.push_str(&ass.to_string(indent + 1));
                    s.push('\n');
                }

                s
            }
        }
    }
}

impl Debug for ASTNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(0))
    }
}

impl ToString for ASTNode {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}