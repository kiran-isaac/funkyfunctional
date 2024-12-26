use crate::{ASTNode, ASTNodeType, AST};
mod replace;
mod reduce;


#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum ProgressType {
    None, 
    Replace,
    Reduce
}

#[derive(Debug)]
pub struct Progress {
    pub redex : usize,
    pub contraction : usize,
    pub progress_type : ProgressType
}

// Get the things in the expression tree that can be replaced by definitions in the module
pub fn get_replacements(expr_ast : &AST, mod_ast : &AST) -> Vec<(usize, usize)> {
    replace::get_replacement_targets(mod_ast, expr_ast, expr_ast.root)
}