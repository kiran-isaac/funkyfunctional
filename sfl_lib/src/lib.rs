mod find_redexes;
mod inbuilts;
mod parser;
mod types;

pub use find_redexes::find_redex_contraction_pairs;
pub use parser::ast::{ASTNode, ASTNodeType, AST};
pub use parser::Parser;
pub use types::{Primitive, Type, TypeChecker, TypeError};

#[cfg(test)]
mod lib_test;
