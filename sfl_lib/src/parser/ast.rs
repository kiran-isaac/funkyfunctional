use std::collections::btree_set::Union;

use super::token::*;

#[derive(Clone)]
pub enum ASTNodeType {
    Identifier,
    Literal,

    Application,
    Assignment,

    Module
} 

#[derive(Clone)]
pub struct ASTNode { 
    t : ASTNodeType,
    info : Option<Token>,
    children : Vec<ASTNode>
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
    pub fn new_id(tk : Token) -> Self {
        Self {t : ASTNodeType::Identifier, info : Some(tk), children : vec![]}
    }

    pub fn new_lit(tk : Token) -> Self {
        Self {t : ASTNodeType::Literal, info : Some(tk), children : vec![]}
    }

    pub fn new_app(f : ASTNode, arg : ASTNode) -> Self {
        Self {t : ASTNodeType::Application, info : None, children : vec![f, arg]}
    }

    pub fn new_assignment(id : Token, exp : ASTNode) -> Self {
        Self {t : ASTNodeType::Assignment, info : None, children : vec![Self::new_id(id), exp]}
    }

    pub fn new_module(children : Vec<ASTNode>) -> Self {
        Self {t : ASTNodeType::Module, info : None, children }
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
}