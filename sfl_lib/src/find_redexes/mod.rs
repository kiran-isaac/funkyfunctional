use crate::{ASTNodeType, AST};
mod reduce;

pub use reduce::{find_all_redex_contraction_pairs, find_single_redex_contraction_pair};
pub type RCPair = (usize, AST);

mod pattern_match;
#[cfg(test)]
mod reduce_test;
