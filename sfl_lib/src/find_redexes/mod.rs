use crate::{ASTNodeType, AST};
mod reduce;

pub use reduce::find_redex_contraction_pairs;
pub type RCPair = (usize, AST);

#[cfg(test)]
mod tests;
