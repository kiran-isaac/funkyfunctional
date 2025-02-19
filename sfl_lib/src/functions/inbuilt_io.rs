use crate::*;

pub fn putstrln(call: &ASTNode, args: Vec<&ASTNode>) -> AST {
    println!("putstrln({:?}, {:?})", call, args);
}