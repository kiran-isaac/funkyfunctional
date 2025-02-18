mod find_redexes;
mod functions;
mod parser;
mod types;
mod prelude;

pub use find_redexes::{
    find_all_redex_contraction_pairs, find_single_redex_contraction_pair, RCPair,
};
pub use functions::KnownTypeLabelTable;
pub use parser::ast::{ASTNode, ASTNodeType, AST};
pub use parser::Parser;
pub use prelude::PRELUDE;
pub use types::{typecheck, typecheck_tl_expr, Primitive, Type, TypeError};

#[cfg(test)]
mod lib_test;
