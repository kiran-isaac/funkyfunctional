use super::token::*;

#[derive(Clone)]
pub enum ASTNodeType {
    Identifier,
    Literal,

    Application,
    Assignment,
} 

#[derive(Clone)]
pub struct ASTNode { 
    t : ASTNodeType,
    children : Vec<ASTNode>
}

impl ASTNode {
    pub fn new(t : ASTNodeType, children : &[ASTNode]) -> ASTNode {
        ASTNode {
            t,
            children: children.to_vec(),
        }
    }
}