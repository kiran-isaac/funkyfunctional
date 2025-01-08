mod find_redexes;
mod functions;
mod parser;
mod types;

pub use find_redexes::find_redex_contraction_pairs;
pub use parser::ast::{ASTNode, ASTNodeType, AST};
pub use parser::Parser;
pub use types::{Type, Primitive, TypeError, typecheck_tl_expr, typecheck_module};

#[cfg(test)]
mod lib_test;
