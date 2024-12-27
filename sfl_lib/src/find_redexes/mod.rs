use crate::{ASTNode, ASTNodeType, AST};
mod reduce;

pub use reduce::find_redex_contraction_pairs;


#[cfg(test)]
mod tests;