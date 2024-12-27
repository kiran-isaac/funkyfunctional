use crate::{ASTNode, ASTNodeType, AST};
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

// // Get the things in the expression tree that can be replaced by definitions in the module
// pub fn get_replacements(ast : &AST, mod_ref : usize, expr_ref : usize) -> Vec<(usize, usize)> {
//     replace::get_replacement_targets(ast, mod_ref, expr_ref)
// }
